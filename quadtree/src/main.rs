mod geom;
mod isoline;

use isoline::*;
use geom::*;

use arrayvec::ArrayVec;
use nalgebra::Vector2;
use sdl2::event::Event;
use sdl2::pixels::Color;
use sdl2::render::Canvas;
use sdl2::video::Window;
use std::boxed::Box;
use std::collections::HashMap;

pub type Index = usize;

#[derive(Default, Debug, Copy, Clone)]
struct Vertex {
    value: bool,
}

// An edge that intersects the implicit surface.
// We only keep track of edges that cross the surface.
#[derive(Clone, Debug)]
struct Edge {
    verts: [Index; 2],
    dual_verts: ArrayVec<[Vector2<f32>; 2]>, // vertices from adjacent faces.
    position: Vector2<f32>, // position of intersection.
    normal: Vector2<f32>,   // normal of surface at point of intersection.
}

#[derive(Clone)]
struct Face {
    verts: [Index; 4],                 // Z ordered
    dual_vertex: Option<Vector2<f32>>, // computed dual vertex if the face is part of the surface
    children: Box<[Option<Face>; 4]>,
}

pub struct HermiteGrid {
    width: u32,
    height: u32,
    verts: Vec<Vertex>,
    edges: HashMap<(Index, Index), Edge>, // Keyed by vertex indices
}

impl HermiteGrid {
    pub fn new(width: u32, height: u32) -> HermiteGrid {
        let verts: Vec<Vertex> = vec![Vertex { value: false }; (width * height) as usize];
        HermiteGrid {
            width,
            height,
            verts,
            edges: HashMap::new(),
        }
    }

    pub fn vertex_position(&self, v: &Index) -> Vector2<f32> {
        let i = v % self.width as usize;
        let j = (v - i) / self.width as usize;
        Vector2::new(i as f32, j as f32)
    }

    pub fn vertex_index(&self, x: u32, y: u32) -> usize {
        (x + y * self.width) as usize
    }

    pub fn vertex_index_to_xy(&self, v: &Index) -> (u32, u32) {
        let x = v % self.width as usize;
        let y = v - x;
        (x as u32, y as u32)
    }

    // Use bisection method to find the intersection of the isoline and an edge.
    fn find_edge_intersection(&self, v1: &Index, v2: &Index, iso: &dyn IsoLine) -> Vector2<f32> {
        let mut aoffset = 0.0;
        let mut boffset = 1.0;
        let mut midoffset = 0.0;
        let vector = (self.vertex_position(&v2) - self.vertex_position(&v1)).normalize();
        let a_is_low = iso.sample(self.vertex_position(&v1)) < 0.0;
        while boffset - aoffset > 0.04 {
            // 0.04 = 1 / 256
            midoffset = (aoffset + boffset) / 2.0;
            let midval = iso.sample(self.vertex_position(&v1) + vector * midoffset);
            if (a_is_low && midval <= 0.0) || (!a_is_low && midval > 0.0) {
                aoffset = midoffset;
            } else {
                boffset = midoffset;
            }
        }
        self.vertex_position(v1) + vector * midoffset
    }

    fn make_edge(&self, v1: Index, v2: Index, iso: &dyn IsoLine) -> Edge {
        let position = self.find_edge_intersection(&v1, &v2, iso);
        Edge {
            verts: [v1, v2],
            dual_verts: ArrayVec::new(),
            position,
            normal: iso.normal(position),
        }
    }

    /// Apply a union operation to the Grid
    pub fn add_contour(&mut self, iso: &dyn IsoLine) {
        for j in 0..self.height {
            for i in 0..self.width {
                let index = self.vertex_index(i, j);
                let position = Vector2::new(i as f32, j as f32);

                // TODO: support multiple values, not just binary.
                self.verts[index].value |= iso.sample(position) > 0.0;

                // Add hermite data to grid.
                if i > 0 {
                    let left_index = self.vertex_index(i - 1, j);
                    if self.verts[left_index].value != self.verts[index].value {
                        self.edges
                            .insert((left_index, index),
                                    self.make_edge(left_index, index, iso));
                    }
                }

                if j > 0 {
                    let up_index = self.vertex_index(i, j - 1);
                    if self.verts[up_index].value != self.verts[index].value {
                        self.edges
                            .insert((up_index, index), 
                                    self.make_edge(up_index, index, iso));
                    }
                }
            }
        }
    }
}

pub struct QuadTree {
    root: Box<Face>,
    grid: HermiteGrid,
}

impl QuadTree {
    /// Width/Height: number of faces across.
    pub fn new(width: u32, height: u32) -> QuadTree {
        if width != height || width.next_power_of_two() != width {
            panic!(
                "Right now, QuadTree only works with width == height and width is a power of two"
            );
        }

        let grid = HermiteGrid::new(width + 1, height + 1);

        QuadTree {
            root: Box::new(Face {
                verts: [
                    0,
                    grid.vertex_index(width, 0),
                    grid.vertex_index(0, height),
                    grid.vertex_index(width, height),
                ],
                dual_vertex: None,
                children: Box::new([None, None, None, None]),
            }),
            grid,
        }
    }

    fn face_vertices(&self, f: &Face) -> [&Vertex; 4] {
        [
            &self.grid.verts[f.verts[0]],
            &self.grid.verts[f.verts[1]],
            &self.grid.verts[f.verts[2]],
            &self.grid.verts[f.verts[3]],
        ]
    }

    /// If a face is homogeneous (same value throughout), get the value.
    /// If the face is not homogeneous, this returns None.
    fn face_homogeneous_value(&self, f: &Face) -> Option<bool> {
        for c in f.children.iter() {
            if !c.is_none() {
                return None; // Assume if we have a child we are not homogeneous
            }
        }

        let verts = self.face_vertices(f);
        let value = verts[0].value;
        for i in 1..verts.len() {
            if verts[i].value != value {
                return None;
            }
        }
        Some(value)
    }

    /// Same as face_homogeneous_value, but checks if all Faces are homogeneous
    /// and share the *same* value with each other.
    fn faces_homogeneous_value(&self, f: &[Option<Face>; 4]) -> Option<bool> {
        let mut value: Option<bool> = None;
        for i in 0..f.len() {
            if f[i].is_some() {
                let face = f[i].as_ref().unwrap();
                if value.is_none() {
                    value = self.face_homogeneous_value(face);
                } else if value != self.face_homogeneous_value(face) {
                    return None;
                }
            }
        }
        return value;
    }

    pub fn build(&mut self) {
        let min = (0, 0);
        let max = (self.grid.width - 1, self.grid.height - 1);

        self.root = Box::new(self.build_face([min, (max.0, min.1), (min.0, max.1), max]));
    }

    pub fn get_contour(&self) -> Vec<((f32, f32), (f32, f32))> {
        let mut v = vec![];
        for e in self.grid.edges.values() {
            if e.dual_verts.len() < 2 { 
                println!("bad edge; or edge crossed boundary");
                continue; 
            }
            v.push(((e.dual_verts[0].x, e.dual_verts[0].y), 
                    (e.dual_verts[1].x, e.dual_verts[1].y)));
        }
        v
    }

    fn build_face(&mut self, corners: [(u32, u32); 4]) -> Face {
        assert_eq!(corners[3].0 - corners[0].0, corners[3].1 - corners[0].1);
        assert_eq!(corners[0].1, corners[1].1);
        assert_eq!(corners[2].1, corners[3].1);
        assert_eq!(corners[0].0, corners[2].0);
        assert_eq!(corners[1].0, corners[3].0);

        let verts = [
            self.grid.vertex_index(corners[0].0, corners[0].1),
            self.grid.vertex_index(corners[1].0, corners[1].1),
            self.grid.vertex_index(corners[2].0, corners[2].1),
            self.grid.vertex_index(corners[3].0, corners[3].1),
        ];

        let mut children: [Option<Face>; 4] = [None, None, None, None];
        let mut dual_vertex = None;

        // if we are not yet at the finest granularity.
        if corners[3].0 - corners[0].0 > 1 {
            let min = (corners[0].0, corners[0].1);
            let max = (corners[3].0, corners[3].1);
            let mid = (
                min.0 + (max.0 - min.0) / 2,
                min.1 + (max.1 - min.1) / 2,
            );

            let top = (mid.0, min.1);
            let left = (min.0, mid.1);
            let right = (max.0, mid.1);
            let bottom = (mid.0, max.1);

            let bottom_left = (min.0, max.1);
            let top_right = (max.0, min.1);

            // Z ordered vertex locations for child face corners.
            for (i, corner_set) in [
                [min, top, left, mid],
                [top, top_right, mid, right],
                [left, mid, bottom_left, bottom],
                [mid, right, bottom, max],
            ]
            .iter()
            .enumerate()
            {
                let child = self.build_face(*corner_set);
                // Only keep children that are heterogeneous.
                if self.face_homogeneous_value(&child).is_none() {
                    children[i] = Some(child);
                }
            }
        } else {
            // Iterate over edges in this face.
            // Check if we need to add a dual vertex.
            let mut edge_keys = vec![];
            for e in [(0, 1), (0, 2), (1, 3), (2, 3)].iter() {
                let edge_key = (verts[e.0], verts[e.1]);
                let edge = self.grid.edges.get(&edge_key);
                // If edge exists, then add it to dual_vertex position.
                if edge.is_some() {
                    // dual_vertex += edge.position
                    let v = dual_vertex.map_or(Vector2::zeros(), |v| v);
                    dual_vertex = Some(v + edge.unwrap().position);
                    edge_keys.push(edge_key);
                }
            }
            dual_vertex = dual_vertex.map(|v| {
                v / edge_keys.len() as f32
            });
            for key in edge_keys.iter() {
                let edge = self.grid.edges.get_mut(&key);
                edge.unwrap().dual_verts.push(dual_vertex.unwrap());
            }
        }

        Face {
            verts,
            dual_vertex,
            children: Box::new(children),
        }
    }
}

fn draw_points(canvas: &mut Canvas<Window>, points: &Vec<(f32, f32)>) {
    for p in points.iter() {
        canvas.draw_point(((p.0 * 20.0) as i32, (p.1 * 20.0) as i32)).expect("bad draw");
    }
}

fn draw_lines(canvas: &mut Canvas<Window>, lines: &Vec<((f32, f32), (f32, f32))>) {
    for l in lines.iter() {
        canvas.draw_line(
            (((l.0).0 * 20.0) as i32, ((l.0).1 * 20.0) as i32),
            (((l.1).0 * 20.0) as i32, ((l.1).1 * 20.0) as i32),
        ).expect("bad draw");
    }
}

fn main() {
    let sdl = sdl2::init().unwrap();
    let video_subsystem = sdl.video().unwrap();
    let window = video_subsystem
        .window("quadtree", 640, 480)
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().build().unwrap();
    let mut event_pump = sdl.event_pump().unwrap();

    let mut qt = QuadTree::new(4, 4);
    let circle = Circle::new(Vector2::new(1.5, 1.5), 1.0);
    qt.grid.add_contour(&circle);
    qt.build();
    let lines = qt.get_contour();

    'main: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'main,
                _ => {}
            }
        }

        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        canvas.set_draw_color(Color::RGB(255, 255, 255));
        draw_lines(&mut canvas, &lines);
        canvas.present();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_circle_to_grid() {
        let mut grid = HermiteGrid::new(5, 5);
        let circle = Circle::new(Vector2::new(1.5, 1.5), 1.0);
        grid.add_contour(&circle);

        let exp_verts = [
            false, false, false, false, false, false, true, true, false, false, false, true, true,
            false, false, false, false, false, false, false, false, false, false, false, false,
        ];
        for (i, v) in grid.verts.iter().enumerate() {
            assert_eq!(v.value, exp_verts[i], "expected: {:?}", grid.verts);
        }

        let exp_edges = [
            (1, 6),
            (2, 7),
            (5, 6),
            (7, 8),
            (10, 11),
            (12, 13),
            (11, 16),
            (12, 17),
        ];
        for e in exp_edges.iter() {
            assert!(grid.edges.get(e).is_some(), "missing edge");
        }

        assert_eq!(exp_edges.len(), grid.edges.len());
    }

    #[test]
    fn test_make_quadtree_from_grid() {
        let mut qt = QuadTree::new(4, 4);
        let circle = Circle::new(Vector2::new(1.5, 1.5), 1.0);
        qt.grid.add_contour(&circle);
        qt.build();
    }
}

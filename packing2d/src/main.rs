extern crate sdl2;
extern crate gl;
extern crate nalgebra;
extern crate rand;

use sdl2::pixels::Color;
use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::event::Event;
use sdl2::rect::Rect;
use nalgebra::Vector2;
use nalgebra::Vector3;
use nalgebra::RowVector3;
use nalgebra::Matrix3;
use nalgebra::zero;

struct Face {
    internal_vertex: Vector2<f32>,
    has_vertex: bool,
    edges: [usize; 4],
}

struct Vert {
    sdf: f32, // negative = internal to the implicit surface.

    // Below could be represented implicitly or something.
    pos: Vector2<f32>,
}

struct Edge {
    pos: Vector2<f32>, // crossing position,
    normal: Vector2<f32>,
    crossed: bool,

    //
    vert_index: [usize; 2],
    face_index: [Option<usize>; 2],
}

struct Grid {
    faces: Vec<Face>,
    verts: Vec<Vert>,
    edges: Vec<Edge>,
}

impl Grid {
    fn new(nverts: usize) -> Grid {
        let mut faces: Vec<Face> = vec![];
        let mut verts: Vec<Vert> = vec![];
        let mut edges: Vec<Edge> = vec![];

        verts.reserve(nverts * nverts);
        edges.reserve(nverts * (nverts - 1) * 2);
        faces.reserve((nverts - 1) * (nverts - 1));

        // verts
        for j in 0..nverts {
            for i in 0..nverts {
                verts.push(Vert{sdf: 0.0, 
                                pos: Vector2::new(i as f32, j as f32)});
            }
        }

        // horizontal edges
        for j in 0..nverts {
            for i in 0..nverts-1 {
                let vert_index = i + nverts * j;
                
                // we might not have a face on either side...
                let top_face_index: Option<usize> = if j > 0 { 
                    Some(i + (nverts - 1) * (j - 1))
                } else { None };

                let bottom_face_index: Option<usize> = if j < nverts - 1 {
                    Some(i + (nverts - 1) * j)
                } else { None };

                edges.push(Edge{pos: Vector2::new(0.0, 0.0), 
                                normal: Vector2::new(0.0, 0.0), 
                                crossed: false,
                                vert_index: [vert_index, vert_index+1],
                                face_index: [top_face_index, bottom_face_index]});
            }
        }

        // vertical edges
        for j in 0..nverts-1 {
            for i in 0..nverts {
                let vert_index = i + nverts * j;

                // we might not have a face on either side...
                let left_face_index: Option<usize> = if i > 0 { 
                    Some(i - 1 + (nverts - 1) * j)
                } else { None };

                let right_face_index: Option<usize> = if i < nverts - 1 {
                    Some(i + (nverts - 1) * j)
                } else { None };

                edges.push(Edge{pos: Vector2::new(0.0, 0.0), 
                                normal: Vector2::new(0.0, 0.0), 
                                crossed: false,
                                vert_index: [vert_index, vert_index+nverts],
                                face_index: [left_face_index, right_face_index]});
            }
        }

        // faces
        for j in 0..nverts-1 {
            for i in 0..nverts-1 {
                let vert_index = i + nverts * j;
                let edge_index = i + (nverts - 1) * j;
                faces.push(Face{internal_vertex: Vector2::new(0.0, 0.0),
                                has_vertex: false,
                                edges: [edge_index, edge_index + nverts - 1,
                                        (nverts * (nverts - 1)) + vert_index, 
                                        (nverts * (nverts - 1) + vert_index + 1)],
                });
            }
        }

        Grid{
            faces: faces,
            verts: verts,
            edges: edges,
        }
    }

    fn apply_sdf(&mut self, sdf: fn(pos: Vector2<f32>) -> f32) {
        for v in &mut self.verts {
            v.sdf = sdf(v.pos);
        }

        // use bisection method to find the intersection of the sdf and each edge.
        for e in &mut self.edges {
            let v1 = &self.verts[e.vert_index[0]];
            let v2 = &self.verts[e.vert_index[1]];
            let vector = (v2.pos - v1.pos).normalize();

            if v1.sdf == 0.0 {
                e.pos = v1.pos;
            } else if v2.sdf == 0.0 {
                e.pos = v2.pos;
            } else if v1.sdf.signum() == v2.sdf.signum() {
                continue;
            } else {
                let mut aoffset = 0.0;
                let mut boffset = 1.0;
                let mut midoffset = 0.0;

                let a_is_low = v1.sdf < 0.0;

                while boffset - aoffset > 0.04 { // 0.04 = 1 / 256
                    midoffset = (aoffset + boffset) / 2.0;
                    let midval = sdf(v1.pos + vector * midoffset);
                    if midval == 0.0 {
                        break;
                        println!("MATCH");
                    }
                    if (a_is_low && (midval < 0.0)) || (!a_is_low && midval > 0.0) {
                        aoffset = midoffset;
                    } else {
                        boffset = midoffset;
                    }
                }
                e.pos = v1.pos + vector * midoffset;
                //println!("MIDPOS: {}", e.pos);
            }

            e.normal = Vector2::new(sdf(e.pos + (Vector2::x() * 0.01)) - sdf(e.pos - (Vector2::x() * 0.01)), 
                                    sdf(e.pos + (Vector2::y() * 0.01)) - sdf(e.pos - (Vector2::y() * 0.01))).normalize();
            e.crossed = true;
        }

        // build face vertices
        // Using method in Garland 1997: Surface Simplification Using Quadric Error Metrics.
        // Basically find the intersection of all of the lines defined by the edge crossing
        // position and edge crossing normal.
        for f in &mut self.faces {
            let mut qef: Matrix3<f32> = zero();
            let mut crossed_edge: Vec<&Edge> = Vec::with_capacity(4);
            for i in 0..4 {
                let edge = &self.edges[f.edges[i]];
                if edge.crossed {
                    crossed_edge.push(&edge);
                    let normal_equation = Vector3::new(edge.normal.x, edge.normal.y, -edge.normal.dot(&edge.pos));
                    qef += normal_equation * normal_equation.transpose();
                }
            }

            if crossed_edge.len() == 1 {
                f.internal_vertex = crossed_edge[0].pos;
                f.has_vertex = true;
            } /*else if crossed_edge.len() > 1 {
                let mat = Matrix3::new(
                    qef[(0, 0)], qef[(0, 1)], qef[(0, 2)],
                    qef[(1, 0)], qef[(1, 1)], qef[(1, 2)],
                    0.0, 0.0, 1.0);
                let maybe_inverse = mat.try_inverse();
                if let Some(inv) = maybe_inverse {
                    let joined_pos = inv * Vector3::new(0.0, 0.0, 1.0);
                    f.internal_vertex = joined_pos.xy();
                    println!("FV: {}", joined_pos);
                    f.has_vertex = true;
                }*/ else {
                    if crossed_edge.len() > 0 {
                        for e in &crossed_edge {
                            f.internal_vertex += e.pos;
                        }
                        f.internal_vertex /= crossed_edge.len() as f32;
                        f.has_vertex = true;
                    //}
                }
            }
        }
    }

    fn draw_edge(&self, canvas: &mut Canvas<Window>, e: &Edge) {
        let v1 = self.verts[e.vert_index[0]].pos * 30.0;
        let v2 = self.verts[e.vert_index[1]].pos * 30.0;
        //println!("A: {}, B: {}", v1, v2);
        canvas.draw_line((v1.x as i32, v1.y as i32),
                         (v2.x as i32, v2.y as i32)).expect("bad line");
    }

    fn draw_line(&self, canvas: &mut Canvas<Window>, v1: Vector2<f32>, v2: Vector2<f32>) {
        let v1i = v1 * 30.0;
        let v2i = v2 * 30.0;
        //println!("A: {}, B: {}", v1, v2);
        canvas.draw_line((v1i.x as i32, v1i.y as i32),
                         (v2i.x as i32, v2i.y as i32)).expect("bad line");
    }

    fn draw_face(&self, canvas: &mut Canvas<Window>, f: &Face) {
        self.draw_edge(canvas, &self.edges[f.edges[0]]);
        self.draw_edge(canvas, &self.edges[f.edges[1]]);
        self.draw_edge(canvas, &self.edges[f.edges[2]]);
        self.draw_edge(canvas, &self.edges[f.edges[3]]);
    }

    fn draw_points(&self, canvas: &mut Canvas<Window>) {
        for v in &self.verts {
            canvas.set_draw_color(Color::RGB(0, 255, 0));
            if v.sdf > 0.0 {
                canvas.set_draw_color(Color::RGB(255, 0, 0));
            }
            canvas.draw_point(((v.pos.x * 30.0) as i32, (v.pos.y * 30.0) as i32)).expect("bad draw");
        }

        canvas.set_draw_color(Color::RGB(255, 255, 255));

        for e in &self.edges {
            if !e.crossed {
                continue;
            }

            let point = e.pos * 30.0;
            canvas.draw_point((point.x as i32, point.y as i32)).expect("bad edge");
            /*
            canvas.draw_line((point.x as i32, point.y as i32),
                             ((point.x + e.normal.x * 5.0) as i32,
                              (point.y + e.normal.y * 5.0) as i32)).expect("bad line");
                              */

            if e.face_index[0].is_some() && e.face_index[1].is_some() {
                canvas.set_draw_color(Color::RGB(255, 255, 255));
                let face1 = &self.faces[e.face_index[0].unwrap()];
                let face2 = &self.faces[e.face_index[1].unwrap()];
                self.draw_line(canvas, face1.internal_vertex, face2.internal_vertex);
            }
        }

        for f in &self.faces {
            //let f = &self.faces[i];
            canvas.set_draw_color(Color::RGB(255, 0, 255));
            if rand::random::<u8>() > 250 {
                //self.draw_face(canvas, f);
            }
            if f.has_vertex {
                canvas.set_draw_color(Color::RGB(255, 0, 255));
                let point = f.internal_vertex * 30.0;
                //canvas.fill_rect(Rect::new(point.x as i32 - 2, point.y as i32 - 2, 4, 4)).expect("bad face");
            }
        }
    }
}

const GRID_SIZE: usize = 40;

fn circle_sdf(p: Vector2<f32>) -> f32 {
    let delta = Vector2::new(5.5, 5.5) - p;
    delta.dot(&delta) - 18.0
}

fn main() {
    let sdl = sdl2::init().unwrap();
    let video_subsystem = sdl.video().unwrap();
    let window = video_subsystem.window("Hello", 640, 480).build().unwrap();
    let mut canvas = window.into_canvas().build().unwrap();
    let mut event_pump = sdl.event_pump().unwrap();

    let mut grid = Grid::new(32);
    grid.apply_sdf(circle_sdf);

    let mut lines = Vec::new();
    for e in &grid.edges {
        if e.crossed {
            lines.push((1, 1));
        }
    }

    'main: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit{..} => break 'main,
                _ => {},
            }
        }

        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();

        grid.draw_points(&mut canvas);
        canvas.present();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grid_indices() {
        let mut grid = Grid::new(4);
        assert_eq!(grid.verts.len(), 16);
        assert_eq!(grid.edges.len(), 24);
        assert_eq!(grid.faces.len(), 9);

        for i in 0..3 {
            assert_eq!(grid.edges[i].face_index[0], None);
        }

        for i in 9..12 {
            assert_eq!(grid.edges[i].face_index[1], None);
        }

        for i in 0..9 {
            assert_eq!(grid.edges[i].face_index[1], Some(i));
        }

        // check horizontal edges
        for (e, t, b) in [(0, None, Some(0)), (1, None, Some(1)), (2, None, Some(2)), 
            (3, Some(0), Some(3)), (4, Some(1), Some(4)), (5, Some(2), Some(5)), 
            (6, Some(3), Some(6)), (7, Some(4), Some(7)), (8, Some(5), Some(8)), 
            (9, Some(6), None), (10, Some(7), None), (11, Some(8), None)].iter() {
            assert_eq!(grid.edges[*e as usize].face_index[0], *t);
            assert_eq!(grid.edges[*e as usize].face_index[1], *b);
        }

        // check vertical edges
        for (e, l, r) in [(12, None, Some(0)), (13, Some(0), Some(1)), 
                (14, Some(1), Some(2)), (15, Some(2), None),
            (16, None, Some(3)), (17, Some(3), Some(4)), 
                (18, Some(4), Some(5)), (19, Some(5), None),
            (20, None, Some(6)), (21, Some(6), Some(7)),
                (22, Some(7), Some(8)), (23, Some(8), None)].iter() {
            assert_eq!(grid.edges[*e as usize].face_index[0], *l);
            assert_eq!(grid.edges[*e as usize].face_index[1], *r);
        }
    }
}

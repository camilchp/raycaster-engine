use sdl2::pixels::Color;
use sdl2::render::{WindowCanvas};
use sdl2::rect::{Rect, Point};
use sdl2::event::{Event};
use sdl2::EventPump;
use sdl2::keyboard::{Keycode};
use std::f32::consts::PI;
use std::{thread, time};

const MAP_LENGTH : usize = 10; // Number of tiles in one row/column
const EXP : i32 = 6; // Tile size is a power of two for fast rounding
const TILE_SIZE : i32 = 1 << EXP;
const P_RAD : i32 = 3; //Player Radius
const WIN_LENGTH : u32 = (MAP_LENGTH as i32 * TILE_SIZE) as u32;
const WIN_WIDTH : u32 = WIN_LENGTH;
const SPEED : f32 = 5.;
const ANGULAR_SPEED : f32 = 0.1;
const SEMI_FOV : f32 = PI/8.; // Half of Player's Field Of View
const MAX_DOF : usize = 9; // Player's Maximum Depth Of Field

const P2 : f32 = PI/2.;
const P3 : f32 = 3.*PI/2.;

fn main() {
    let map = [[1,1,1,1,1,1,1,1,1,1],
            [1,0,0,1,0,0,0,0,0,1],
            [1,0,0,1,0,0,0,0,0,1],
            [1,0,0,1,0,0,0,0,0,1],
            [1,0,0,0,0,0,0,0,0,1],
            [1,0,0,0,0,0,0,0,0,1],
            [1,0,0,0,0,0,0,0,0,1],
            [1,0,0,0,0,0,0,1,0,1],
            [1,0,0,0,0,0,0,0,0,1],
            [1,1,1,1,1,1,1,1,1,1], ];

            println!("{}", WIN_LENGTH);

    let mut px : i32 =( WIN_LENGTH as i32)/2; // Player X
    let mut py : i32 =( WIN_LENGTH as i32)/2; // Player Y
    let mut pa : f32 = 0.; // Player Angle

    let mut pdx : i32 = 0;
    let mut pdy : i32 = 0;
    let mut pda : f32 = 0.;


    let (mut canvas, mut event_pump) = init();


    'running:loop {
        if let Some(event) = event_pump.poll_event() {
            match event {
            
                Event::Quit {..} | Event::KeyDown { keycode : Some(Keycode::Escape) , ..} => {break 'running},

                Event::KeyDown { keycode : Some(Keycode::Q), .. } => {pda = -ANGULAR_SPEED;},

                Event::KeyDown { keycode : Some(Keycode::D), .. } => {pda = ANGULAR_SPEED;},

                Event::KeyDown { keycode : Some(Keycode::Z), .. } => {pdx = (pa.cos()*SPEED).floor() as i32; pdy = (pa.sin()*SPEED).floor() as i32 ;},

                Event::KeyDown { keycode : Some(Keycode::S), .. } => {pdx = - (pa.cos()*SPEED).floor() as i32 ; pdy = - (pa.sin()*SPEED).floor() as i32 ;},

                Event::KeyUp { keycode : Some(Keycode::Q), .. } | Event::KeyUp { keycode : Some(Keycode::D), .. } => {pda = 0.;},

                Event::KeyUp { keycode : Some(Keycode::Z), .. } | Event::KeyUp { keycode : Some(Keycode::S), .. } => {pdx = 0; pdy = 0;},

                _ => {}
            }
        }
        let (i, j) = player_tile(px + pdx + if pdx >= 0 {P_RAD} else {-P_RAD}, py + pdy + if pdy >= 0 {P_RAD} else {-P_RAD});
        if tile_full(&map, i, j) {pdx = 0; pdy = 0}


        pa += pda;
        if pa >= 2.*PI {pa -= 2.*PI;} else if pa < 0. {pa += 2.*PI;}
        px += pdx;
        py += pdy;
        canvas.set_draw_color(Color::RGB(50, 50, 50));
        canvas.clear();
        render_walls(&mut canvas, px, py, pa, &map);
        canvas.present();

        thread::sleep(time::Duration::from_millis(20));
    }
}

fn init() -> (WindowCanvas, EventPump) {
    let sdl_context = sdl2::init().expect("Failed to create sdl context");
    let video_subsystem = sdl_context.video().expect("Failed to create video subsystem"); 
    let window = video_subsystem.window("Raycaster", WIN_LENGTH, WIN_WIDTH)
        .position_centered()
        .build()
        .expect("Failed to create window");
    let event_pump = sdl_context.event_pump().unwrap();
    let canvas = window.into_canvas().build().expect("Failed to create canvas");
    (canvas, event_pump)


}

//-----------UTILITY-----------------

fn tile_full(map : &[[u8;MAP_LENGTH];MAP_LENGTH], i : usize, j : usize) -> bool {
    !(map[i][j] == 0)
}

fn player_tile(px : i32, py : i32) -> (usize, usize) {
    let j = (px >> EXP) as usize; // Current tile column
    let i = (py >> EXP) as usize; // Curent tile lign
    (i, j)
}

//-----------render--------

fn render_walls(canvas : &mut WindowCanvas, px : i32, py : i32, pa : f32, map : &[[u8;MAP_LENGTH];MAP_LENGTH]) {

    //===================SHOOT RAYS=================

    let mut ra = pa - SEMI_FOV; // Ray Angle

    for rda in 0..WIN_LENGTH {
        ra += 2.*SEMI_FOV/(WIN_LENGTH as f32);
        if ra >= 2.*PI {ra -= 2.*PI;} else if ra < 0. {ra += 2.*PI;}

        //-----Check Horizontal Intersects-----
        let mut dof = 0;
        let mut dist_h = WIN_LENGTH as f32; // distance to horizontal intersect
        let rec_tan = -1./ra.tan();

        let mut ry = 0; // Y of first intersect
        let mut rx = 0; // X of first intersect
        let mut y_step = 0; // Step in Y for next intersect
        let mut x_step = 0; // Step in X for next intersect

        if ra.abs() <= 0.00001 || (ra - PI).abs() <= 0.00001 {ry = py; rx = px; dof = MAX_DOF} // looking strait right or left => No horisontal intersect
        else if ra > PI {ry = ((py >> EXP) << EXP )-1; rx = ((((py-ry) as f32)*rec_tan).floor()) as i32 + px ; y_step = -TILE_SIZE; x_step = ((-y_step as f32) * rec_tan).floor() as i32;} // looking Up
        else if ra < PI { ry = ((py >> EXP) << EXP) + TILE_SIZE; rx = ((((py-ry) as f32)*rec_tan).floor()) as i32 + px; y_step = TILE_SIZE; x_step = ((-y_step as f32) * rec_tan).floor() as i32;} // looking Down

        while dof < MAX_DOF {
            //draw_player(canvas, rx, ry, ra);
            let i = (ry >> EXP) as usize; // lign of intersected tile
            let j = (rx >> EXP) as usize; // column of intersected tile
            if i >= 0 && j >=0 && i < MAP_LENGTH && j < MAP_LENGTH && tile_full(map, i, j) {dof = MAX_DOF; dist_h = (((px-rx)*(px-rx) + (py - ry)*(py - ry)) as f32).sqrt();}
            else {ry += y_step; rx += x_step; dof += 1;} // Go to next intersect
        }
        
        //----Check Vertical Intersects-----
        let mut dof = 0;
        let mut dist_v = WIN_LENGTH as f32; // distance to vertical intersect
        let neg_tan = -1.*ra.tan();

        let mut ry = 0; // Y of first intersect
        let mut rx = 0; // X of first intersect
        let mut y_step = 0; // Step in Y for next intersect
        let mut x_step = 0; // Step in X for next intersect

        if ra == P2 || ra == P3 {ry = py; rx = px; dof = MAX_DOF} // looking strait up or down => No vertical intersect
        else if ra > P2 && ra < P3 {rx = ((px >> EXP) << EXP) -1; ry = ((((px-rx) as f32)*neg_tan).floor()) as i32 + py ; x_step = -TILE_SIZE; y_step = ((-x_step as f32) * neg_tan).floor() as i32;} // looking left
        else if ra < P2 || ra > P3 { rx = ((px >> EXP) << EXP) + TILE_SIZE; ry = ((((px-rx) as f32)*neg_tan).floor()) as i32 + py; x_step = TILE_SIZE; y_step = ((-x_step as f32) * neg_tan).floor() as i32;} // looking right

        while dof < MAX_DOF {
            //draw_player(canvas, rx, ry, ra);
            let i = (ry >> EXP) as usize; // lign of intersected tile
            let j = (rx >> EXP) as usize; // column of intersected tile
            if i >= 0 && j >=0 && i < MAP_LENGTH && j < MAP_LENGTH && tile_full(map, i, j) {dof = MAX_DOF; dist_v = (((px-rx)*(px-rx) + (py - ry)*(py - ry)) as f32).sqrt();} else {ry += y_step; rx += x_step; dof += 1;} // Go to next intersect
        }

        let dist = if dist_v < dist_h {dist_v} else {dist_h};

        //========Draw Wall===========

        let angle_diff = pa - ra;
        let dist_no_fisheye = dist * angle_diff.cos();

        let mut line_height = ((WIN_WIDTH as f32)*100./dist_no_fisheye).floor() as u32;
        if  line_height > WIN_WIDTH {line_height = WIN_WIDTH;}
        
        let line_y = (WIN_WIDTH/2 - line_height/2) as i32;

        if dist_v < dist_h {canvas.set_draw_color(Color::RGB(200, 200, 200));} else {canvas.set_draw_color(Color::RGB(150, 150, 150));}
        canvas.fill_rect(Rect::new(rda as i32, line_y, 1, line_height));
        

    }
}


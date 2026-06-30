use std::cmp::{max, min};

use image::GrayImage;
use x11rb::{
  connection::Connection,
  protocol::{
    Event,
    xproto::{
      ChangeWindowAttributesAux, ConfigureWindowAux, ConnectionExt,
      CreateGCAux, CreateWindowAux, EventMask, GrabMode, ImageFormat, Pixmap,
      Window,
    },
  },
  rust_connection::RustConnection,
};

pub struct X11Backend {
  conn: RustConnection,
  screen_num: usize,
}

impl X11Backend {
  pub fn new() -> anyhow::Result<Self> {
    let (conn, screen_num) = RustConnection::connect(None)?;
    Ok(Self { conn, screen_num })
  }
}

impl super::Backend for X11Backend {
  fn capture(&self) -> anyhow::Result<Option<GrayImage>> {
    let region = match select_region(&self.conn, self.screen_num)? {
      Some(r) => r,
      None => return Ok(None),
    };
    Ok(Some(screenshot(&self.conn, self.screen_num, &region)?))
  }
}

struct Region {
  x: i16,
  y: i16,
  width: u16,
  height: u16,
}

fn select_region(
  conn: &RustConnection,
  screen_num: usize,
) -> anyhow::Result<Option<Region>> {
  let screen = &conn.setup().roots[screen_num];
  let root = screen.root;
  let sw = screen.width_in_pixels;
  let sh = screen.height_in_pixels;
  let depth = screen.root_depth;
  let visual = screen.root_visual;

  let bg_pixmap = snapshot_to_pixmap(conn, root, sw, sh, depth)?;

  let win: Window = conn.generate_id()?;
  conn.create_window(
    depth,
    win,
    root,
    0,
    0,
    sw,
    sh,
    0,
    x11rb::protocol::xproto::WindowClass::INPUT_OUTPUT,
    visual,
    &CreateWindowAux::new()
      .event_mask(
        EventMask::BUTTON_PRESS
          | EventMask::BUTTON_RELEASE
          | EventMask::POINTER_MOTION
          | EventMask::KEY_PRESS
          | EventMask::EXPOSURE,
      )
      .override_redirect(1)
      .background_pixmap(bg_pixmap),
  )?;

  conn.configure_window(
    win,
    &ConfigureWindowAux::new()
      .stack_mode(x11rb::protocol::xproto::StackMode::ABOVE),
  )?;

  conn.change_window_attributes(
    win,
    &ChangeWindowAttributesAux::new().cursor(make_crosshair(conn)?),
  )?;

  conn.map_window(win)?;
  conn.flush()?;

  conn.grab_pointer(
    false,
    win,
    EventMask::BUTTON_PRESS
      | EventMask::BUTTON_RELEASE
      | EventMask::POINTER_MOTION,
    GrabMode::ASYNC,
    GrabMode::ASYNC,
    win,
    x11rb::NONE,
    x11rb::CURRENT_TIME,
  )?;
  conn.flush()?;

  let gc = conn.generate_id()?;
  conn.create_gc(
    gc,
    win,
    &CreateGCAux::new()
      .foreground(0x00_FF_FF_FF)
      .function(x11rb::protocol::xproto::GX::XOR)
      .subwindow_mode(x11rb::protocol::xproto::SubwindowMode::INCLUDE_INFERIORS)
      .line_width(2),
  )?;

  let border_gc = conn.generate_id()?;
  conn.create_gc(
    border_gc,
    win,
    &CreateGCAux::new().foreground(0x00_AA_00_00).line_width(1),
  )?;
  draw_rect(conn, win, border_gc, 0, 0, sw - 1, sh - 1)?;
  conn.flush()?;

  let mut start: Option<(i16, i16)> = None;
  let mut last_rect: Option<(i16, i16, u16, u16)> = None;
  let mut result: Option<Region> = None;

  loop {
    let ev = conn.wait_for_event()?;
    match ev {
      Event::ButtonPress(e) if e.detail == 1 => {
        start = Some((e.root_x, e.root_y));
      }
      Event::MotionNotify(e) => {
        if let Some((sx, sy)) = start {
          if let Some((rx, ry, rw, rh)) = last_rect {
            draw_rect(conn, win, gc, rx, ry, rw, rh)?;
          }
          let (rx, ry, rw, rh) = rect_from_points(sx, sy, e.root_x, e.root_y);
          draw_rect(conn, win, gc, rx, ry, rw, rh)?;
          last_rect = Some((rx, ry, rw, rh));
          conn.flush()?;
        }
      }
      Event::ButtonRelease(e) if e.detail == 1 => {
        if let Some((sx, sy)) = start {
          let (rx, ry, rw, rh) = rect_from_points(sx, sy, e.root_x, e.root_y);
          if rw > 2 && rh > 2 {
            result = Some(Region {
              x: rx,
              y: ry,
              width: rw,
              height: rh,
            });
          }
        }
        break;
      }
      Event::KeyPress(e) => {
        if e.detail == 9 {
          break;
        }
      }
      Event::Expose(_) => {
        draw_rect(conn, win, border_gc, 0, 0, sw - 1, sh - 1)?;
        if let Some((rx, ry, rw, rh)) = last_rect {
          draw_rect(conn, win, gc, rx, ry, rw, rh)?;
        }
        conn.flush()?;
      }
      _ => {}
    }
  }

  conn.ungrab_pointer(x11rb::CURRENT_TIME)?;
  conn.destroy_window(win)?;
  conn.free_pixmap(bg_pixmap)?;
  conn.flush()?;

  Ok(result)
}

fn snapshot_to_pixmap(
  conn: &RustConnection,
  root: Window,
  sw: u16,
  sh: u16,
  depth: u8,
) -> anyhow::Result<Pixmap> {
  let img = conn
    .get_image(ImageFormat::Z_PIXMAP, root, 0, 0, sw, sh, !0)?
    .reply()?;

  let mut data = img.data;
  for chunk in data.chunks_mut(4) {
    chunk[0] = (chunk[0] as f32 * 0.85) as u8;
    chunk[1] = (chunk[1] as f32 * 0.85) as u8;
    chunk[2] = (chunk[2] as f32 * 0.85) as u8;
  }

  let pixmap: Pixmap = conn.generate_id()?;
  conn.create_pixmap(depth, pixmap, root, sw, sh)?;

  let gc = conn.generate_id()?;
  conn.create_gc(gc, pixmap, &CreateGCAux::new())?;

  conn.put_image(
    ImageFormat::Z_PIXMAP,
    pixmap,
    gc,
    sw,
    sh,
    0,
    0,
    0,
    depth,
    &data,
  )?;

  conn.free_gc(gc)?;
  Ok(pixmap)
}

fn screenshot(
  conn: &RustConnection,
  screen_num: usize,
  region: &Region,
) -> anyhow::Result<GrayImage> {
  let root = conn.setup().roots[screen_num].root;

  let reply = conn
    .get_image(
      ImageFormat::Z_PIXMAP,
      root,
      region.x,
      region.y,
      region.width,
      region.height,
      !0,
    )?
    .reply()?;

  let data = reply.data;
  let w = region.width as u32;
  let h = region.height as u32;
  let mut gray = GrayImage::new(w, h);

  for (i, pixel) in gray.pixels_mut().enumerate() {
    let base = i * 4;
    let b = data[base] as u32;
    let g = data[base + 1] as u32;
    let r = data[base + 2] as u32;
    let luma = (r * 299 + g * 587 + b * 114) / 1000;
    *pixel = image::Luma([luma as u8]);
  }

  Ok(gray)
}

fn rect_from_points(
  x1: i16,
  y1: i16,
  x2: i16,
  y2: i16,
) -> (i16, i16, u16, u16) {
  let rx = min(x1, x2);
  let ry = min(y1, y2);
  let rw = max(x1, x2) - rx;
  let rh = max(y1, y2) - ry;
  (rx, ry, rw as u16, rh as u16)
}

fn draw_rect(
  conn: &RustConnection,
  win: Window,
  gc: u32,
  x: i16,
  y: i16,
  w: u16,
  h: u16,
) -> anyhow::Result<()> {
  conn.poly_rectangle(
    win,
    gc,
    &[x11rb::protocol::xproto::Rectangle {
      x,
      y,
      width: w,
      height: h,
    }],
  )?;
  Ok(())
}

fn make_crosshair(conn: &RustConnection) -> anyhow::Result<u32> {
  let font = conn.generate_id()?;
  conn.open_font(font, b"cursor")?;
  let cursor = conn.generate_id()?;
  conn.create_glyph_cursor(
    cursor, font, font, 34, // XC_crosshair
    35, 0, 0, 0, 0xFF_FF, 0xFF_FF, 0xFF_FF,
  )?;
  conn.close_font(font)?;
  Ok(cursor)
}

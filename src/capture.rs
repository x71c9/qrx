use std::cmp::{max, min};

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

pub struct Region {
  pub x: i16,
  pub y: i16,
  pub width: u16,
  pub height: u16,
}

pub fn select_region(
  conn: &RustConnection,
  screen_num: usize,
) -> anyhow::Result<Option<Region>> {
  let screen = &conn.setup().roots[screen_num];
  let root = screen.root;
  let sw = screen.width_in_pixels;
  let sh = screen.height_in_pixels;
  let depth = screen.root_depth;
  let visual = screen.root_visual;

  // Snapshot the screen before opening the overlay so the user can see it
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

  // XOR GC for rubber-band rectangle: drawing twice erases
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

  // Darken each pixel by 10% so the overlay reads as slightly dimmed
  let mut data = img.data;
  for chunk in data.chunks_mut(4) {
    chunk[0] = (chunk[0] as f32 * 0.8) as u8; // B
    chunk[1] = (chunk[1] as f32 * 0.8) as u8; // G
    chunk[2] = (chunk[2] as f32 * 0.8) as u8; // R
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

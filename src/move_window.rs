use swayipc::{Connection, Rect};

fn get_workspace_rect() -> Result<Rect, Box<dyn std::error::Error>> {
  let mut connection = Connection::new()?;
  let workspaces = connection.get_workspaces()?;
  let workspace = workspaces
    .iter()
    .find(|workspace| workspace.focused)
    .ok_or("No focused workspace")?;
  Ok(workspace.rect)
}

const WINDOW_CRITERIA: &str = "[app_id=\"dogky\"]";

pub fn move_window(window_width: u32) -> Result<(), Box<dyn std::error::Error>> {
  let mut connection = Connection::new()?;
  let outputs = connection.get_outputs()?;
  let output = outputs.iter().find(|output| output.focused).ok_or("No output")?;
  let current_mode = output.current_mode.ok_or("No current mode")?;
  let scale = output.scale.ok_or("No scale")?;
  let [output_width, output_height] = [
    (current_mode.width as f64 / scale).round() as i32,
    (current_mode.height as f64 / scale).round() as i32,
  ];
  let workspace_rect = get_workspace_rect()?;
  let bars_height = output_height - workspace_rect.height; // Possibly 0

  let inner_height = output_height - bars_height;
  let [pos_x, pos_y] = [output_width - window_width as i32, bars_height];

  connection.run_command(
    [
      format!(
        "for_window {} resize set {} {}",
        WINDOW_CRITERIA, window_width, inner_height
      ),
      format!(
        "for_window {} move absolute position {} {}",
        WINDOW_CRITERIA, pos_x, pos_y
      ),
    ]
    .join(";"),
  )?;
  Ok(())
}

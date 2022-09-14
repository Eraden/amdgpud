use amdgpu_config::fan::MatrixPoint;
use tui::buffer::Buffer;
use tui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use tui::style::{Color, Style};
use tui::symbols::Marker;
use tui::text::{Span, Text};
use tui::widgets::{Axis, Block, Chart, Dataset, List, ListItem, Paragraph, Widget};

use crate::Dir;

#[derive(Default, Debug)]
pub struct FanSpeedState {
    highlighted_point: Option<usize>,
    selected_point: Option<usize>,
    len: usize,
    buffer: String,
}

impl FanSpeedState {
    pub fn move_dir(&mut self, dir: Dir) {
        if self.selected_point.is_some() {
            return;
        }
        match dir {
            Dir::Left => match self.highlighted_point {
                None => self.highlighted_point = Some(0),
                Some(0) => self.highlighted_point = Some(0),
                Some(idx) => self.highlighted_point = Some(idx - 1),
            },
            Dir::Right => match self.highlighted_point {
                None => self.highlighted_point = Some(0),
                Some(idx) if idx + 1 < self.len => self.highlighted_point = Some(idx + 1),
                Some(_) => {}
            },
            Dir::Up => {}
            Dir::Down => {}
        }
    }

    pub fn toggle_select(&mut self) {
        match (self.selected_point, self.highlighted_point) {
            (_, None) => self.selected_point = None,
            (Some(selected), Some(highlighted)) => {
                if selected != highlighted {
                    self.selected_point = Some(highlighted);
                } else {
                    self.selected_point = None;
                }
            }
            (_, Some(highlighted)) => {
                self.selected_point = Some(highlighted);
            }
        }
    }

    pub fn text_input(&mut self, c: char) {
        self.buffer.push(c);
    }

    pub fn erase_last(&mut self) {
        let len = self.buffer.len();
        if len > 0 {
            self.buffer.remove(len - 1);
        }
    }
}

#[derive(Default, Debug)]
pub struct FanSpeed {
    y_axis: Vec<f64>,
    x_axis: Vec<f64>,
    dataset: Vec<(f64, f64)>,
    highlighted_point: Option<usize>,
    selected_point: Option<usize>,
}

impl FanSpeed {
    pub fn with_values(mut self, matrix: &[MatrixPoint]) -> Self {
        self.update_values(matrix);
        self
    }

    pub fn update_values(&mut self, matrix: &[MatrixPoint]) {
        self.x_axis = matrix.iter().map(|point| point.temp).collect::<Vec<_>>();

        self.y_axis = matrix.iter().map(|point| point.speed).collect::<Vec<_>>();

        self.dataset = self
            .x_axis
            .iter()
            .copied()
            .zip(self.y_axis.iter().copied())
            .collect::<Vec<_>>();
    }

    pub fn with_state(mut self, state: &mut FanSpeedState) -> Self {
        self.highlighted_point = state.highlighted_point;
        self.selected_point = state.selected_point;

        state.len = self.x_axis.len();

        self
    }
}

impl Widget for FanSpeed {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let vertical = Layout::default()
            .constraints([Constraint::Length(21), Constraint::Length(1)])
            .direction(Direction::Vertical)
            .split(area);

        let top_horizontal = Layout::default()
            .constraints([Constraint::Length(21), Constraint::Min(10)])
            .direction(Direction::Horizontal)
            .split(vertical[0]);

        let bottom_horizontal = Layout::default()
            .constraints([Constraint::Min(10), Constraint::Min(4), Constraint::Min(10)])
            .direction(Direction::Horizontal)
            .split(vertical[1]);

        tracing::debug!("=======================================================");
        tracing::debug!("vertical {:?}", vertical);
        tracing::debug!("horizontal {:?}", top_horizontal);

        let mut dataset_list = Vec::with_capacity(3);

        match self.highlighted_point {
            Some(idx) => {
                if idx != 0 {
                    dataset_list.push(
                        Dataset::default()
                            .data(&self.dataset[0..idx])
                            .style(Style::default().fg(Color::White)),
                    );
                }

                dataset_list.push(
                    Dataset::default()
                        .data(&self.dataset[idx..=idx])
                        .style(Style::default().fg(Color::Red))
                        .marker(Marker::Dot),
                );

                if idx + 1 < self.x_axis.len() {
                    dataset_list.push(
                        Dataset::default()
                            .data(&self.dataset[(idx + 1)..])
                            .style(Style::default().fg(Color::White)),
                    );
                }
            }
            None => {
                dataset_list.push(
                    Dataset::default()
                        .data(self.dataset.as_slice())
                        .style(Style::default().fg(Color::White)),
                );
            }
        };

        Chart::new(dataset_list)
            .block(Block::default())
            .x_axis(axis("Temperature"))
            .y_axis(axis("Speed"))
            .render(top_horizontal[0], buf);

        List::new(self.dataset.iter().enumerate().fold(
            vec![ListItem::new("Temperature | Speed".to_string())],
            |mut v, (idx, (temperature, speed))| {
                if self.selected_point == Some(idx) {
                    let text = Text::styled(
                        format!("{:>11} | {:>5} *", temperature, speed),
                        Style::default().fg(Color::Yellow),
                    );
                    v.push(ListItem::new(text));
                } else if self.highlighted_point == Some(idx) {
                    v.push(
                        ListItem::new(format!("{:>11} | {:>5} <", temperature, speed))
                            .style(Style::default().fg(Color::Yellow)),
                    );
                } else {
                    v.push(ListItem::new(format!("{:>11} | {:>5}", temperature, speed)));
                }
                v
            },
        ))
        .render(top_horizontal[1], buf);

        Paragraph::new("Selected:").render(bottom_horizontal[0], buf);

        if let Some((x, _y)) = self.selected_point.and_then(|idx| self.dataset.get(idx)) {
            Paragraph::new(format!("{:<3}", x))
        } else {
            Paragraph::new("none")
        }
        .render(bottom_horizontal[1], buf);
    }
}

fn axis(label: &str) -> Axis {
    Axis::default()
        .labels_alignment(Alignment::Center)
        .title(label)
        .bounds([0f64, 101f64])
        .labels(
            (0..=100)
                .into_iter()
                .step_by(10)
                .map(|f| Span::raw(format!("{:^3}", f)))
                .collect::<Vec<_>>(),
        )
}

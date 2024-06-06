use plotters::coord::Shift;

use plotters::{define_color, prelude::*};

use plotters::doc;

define_color!(BLUE, 0, 0, 255, "Blue");

pub fn plot_histogram<DB>(
    entries: impl ToOwned<Owned = Vec<u32>>,
    category_formatter: &impl Fn(usize) -> String,
    caption: &str,
    drawing_area: &DrawingArea<DB, Shift>,
) -> Result<(), Box<dyn std::error::Error>>
where
    DB: DrawingBackend,
    <DB as DrawingBackend>::ErrorType: 'static,
{
    let mut entries = entries.to_owned();
    entries.sort();

    const NUM_STEPS: usize = 15;

    let mut left = ChartBuilder::on(&drawing_area);

    let min = *entries.first().unwrap_or(&0);
    let max = *entries.last().unwrap_or(&0);

    let step = (max - min) / NUM_STEPS as u32;

    let mut chart_context_left = left
        .caption(caption, ("Arial", 20))
        .set_all_label_area_size(50)
        .margin(50)
        .build_cartesian_2d((min..max).step(step).use_round().into_segmented(), 0..20u32)?;

    let label_formatter = |idx: &SegmentValue<u32>| match *idx {
        SegmentValue::Exact(_) => "N/A".to_string(),
        SegmentValue::CenterOf(price) => format!("{} PLN", price / 100),
        SegmentValue::Last => "N/A".to_string(),
    };

    chart_context_left
        .configure_mesh()
        .disable_x_mesh()
        .x_label_formatter(&label_formatter)
        .x_labels(15)
        .y_labels(20)
        .draw()?;

    chart_context_left.draw_series(
        Histogram::vertical(&chart_context_left)
            .style(BLUE.filled())
            .data(entries.into_iter().map(|x| (SegmentValue::CenterOf(x), 1))),
    )?;

    Ok(())
}

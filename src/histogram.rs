use plotters::coord::Shift;

use plotters::prelude::*;

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

    let mut left = ChartBuilder::on(&drawing_area);

    let mut chart_context_left = left
        .caption(caption, ("Arial", 20))
        .set_all_label_area_size(50)
        .margin(50)
        .build_cartesian_2d(
            (0..1000000u32).step(50000).use_round().into_segmented(),
            0..20u32,
        )?;

    let label_formatter = |idx: &SegmentValue<u32>| match *idx {
        SegmentValue::Exact(_) => "N/A".to_string(),
        SegmentValue::CenterOf(price) => format!("{} PLN", price / 100),
        SegmentValue::Last => "N/A".to_string(),
    };

    chart_context_left
        .configure_mesh()
        .disable_x_mesh()
        .x_label_formatter(&label_formatter)
        .draw()?;

    chart_context_left.draw_series(
        Histogram::vertical(&chart_context_left)
            .data(entries.into_iter().map(|x| (SegmentValue::CenterOf(x), 1))),
    )?;

    Ok(())
}

use color_eyre::Result;
use plotters::prelude::*;
use polars::prelude::*;
use reqwest::blocking::Client;
use std::{error::Error, io::Cursor};

const OUT_FILE_NAME: &'static str = "imgs/scatter.png";

fn main() -> Result<()> {
    // データの取得
    let data: Vec<u8> = Client::new()
        .get("https://j.mp/iriscsv")
        .send()?
        .text()?
        .bytes()
        .collect();

    let df = CsvReader::new(Cursor::new(data))
        .has_header(true) //
        .finish()?;

    let df2 = df
        .clone()
        .lazy()
        .filter(col("sepal_length").gt(5))
        .groupby([col("species")])
        .agg([col("*").sum()])
        .collect()?;

    println!("{:?}", df2);
    let root = BitMapBackend::new(OUT_FILE_NAME, (1024, 768)).into_drawing_area();
    root.fill(&WHITE)?;

    scatter(
        &df,
        "sepal_length",
        "petal_length",
        "Sepal Length-Petal Length",
        root,
    )
    .unwrap();

    Ok(())
}

fn scatter<S: AsRef<str>, DB: DrawingBackend>(
    df: &DataFrame,
    x: S,
    y: S,
    caption: S,
    b: DrawingArea<DB, plotters::coord::Shift>,
) -> Result<(), Box<dyn Error>>
where
    DB::ErrorType: 'static,
{
    let x_series = df.column(x.as_ref())?;
    let y_series = df.column(y.as_ref())?;

    let caption_style = TextStyle::from(("sans-serif", 20).into_font());
    let mut scatter_context = ChartBuilder::on(&b)
        .caption(caption, caption_style)
        .margin(7)
        .set_left_and_bottom_label_area_size(40)
        .build_cartesian_2d(as_range(x_series), as_range(y_series))
        .unwrap();
    scatter_context
        .configure_mesh()
        .x_desc(x.as_ref())
        .y_desc(y.as_ref())
        .draw()?;
    scatter_context.draw_series(x_series.iter().zip(y_series.iter()).map(|(x, y)| {
        Circle::new(
            (x.try_extract().unwrap(), y.try_extract().unwrap()),
            5.0,
            GREEN.filled(),
        )
    }))?;

    b.present()?;
    Ok(())
}

fn as_range(s: &Series) -> std::ops::Range<f64> {
    std::ops::Range {
        start: s.min().unwrap(),
        end: s.max().unwrap(),
    }
}

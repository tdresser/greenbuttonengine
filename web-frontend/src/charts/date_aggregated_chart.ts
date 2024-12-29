import * as d3 from 'd3';
import { effect, Signal } from '@preact/signals';
import { DateAggregatedChartRenderer } from './date_aggregated_chart_renderer';
import { TimeSeries } from '../../../lib/wasm/pkg/wasm';

function dateExtent(col: Date[]): [Date, Date] {
  let maybeExtent = d3.extent(col as Date[]);
  if (maybeExtent[0] == undefined || maybeExtent[1] == undefined) {
    throw 'Invalid dates';
  }
  return maybeExtent;
}

function numericExtent(col: d3.TypedArray): [number, number] {
  let maybeExtent = d3.extent(col);
  if (maybeExtent[0] == undefined || maybeExtent[1] == undefined) {
    throw 'Invalid numbers';
  }
  return maybeExtent;
}

// Splitting out the renderers lets us avoid recreating them repeatedly.
export class DateAggregatedChart {
  renderer: DateAggregatedChartRenderer;
  xScale: d3.ScaleTime<number, number, never>;
  yScale: d3.ScaleLinear<number, number, never>;

  constructor(props: {
    renderer: DateAggregatedChartRenderer;
    mainChartsEl: HTMLElement;
    chartBoxesEl: HTMLElement;
    index: number;
    transform: Signal<d3.ZoomTransform>;
    timeseries: TimeSeries;
    values: d3.TypedArray;
    title: string;
    uomOverride?: string;
  }) {
    this.renderer = props.renderer;
    this.renderer.clear();
    let dates = props.timeseries.time_period_start;

    if (dates.length < 2) {
      this.xScale = d3.scaleTime();
      this.yScale = d3.scaleLinear();
      return;
    }
    const uom = props.uomOverride || props.timeseries.uom[0];

    this.xScale = d3
      .scaleTime()
      .domain(dateExtent(dates))
      .nice()
      .range([0, DateAggregatedChartRenderer.WIDTH()]);
    this.yScale = d3
      .scaleLinear()
      .domain([0, numericExtent(props.values)[1]])
      .range([DateAggregatedChartRenderer.HEIGHT(), 0]);

    this.renderer.init({
      mainChartsEl: props.mainChartsEl,
      chartBoxesEl: props.chartBoxesEl,
      dates: dates,
      values: props.values,
      title: props.title,
      uom: uom,
      xScale: this.xScale,
      yScale: this.yScale,
      index: props.index,
    });

    effect(() => {
      this.renderer.draw(props.transform.value, this.xScale, this.yScale);
    });
  }
}

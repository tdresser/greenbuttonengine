import { Signal, effect, signal } from '@preact/signals';
import * as d3 from 'd3';
import { DateAggregatedChart } from './date_aggregated_chart';
import { DateAggregatedChartRenderer } from './date_aggregated_chart_renderer';
import { TimeSeries } from '../../../lib/wasm/pkg/wasm';

export class DateAggregatedCharts {
  renderers: DateAggregatedChartRenderer[] = [];

  constructor(
    chartsEl: HTMLElement,
    chartZoomboxesEl: HTMLElement,
    timeseriesArray: Signal<TimeSeries[]>,
  ) {
    const transform = signal(new d3.ZoomTransform(1, 0, 0));

    effect(() => {
      let index = 0;
      for (const timeseries of timeseriesArray.value) {
        const title = timeseries.title[0];

        new DateAggregatedChart({
          renderer: this.getRenderer(index),
          mainChartsEl: chartsEl,
          chartBoxesEl: chartZoomboxesEl,
          index: index++,
          transform,
          timeseries,
          title,
          values: timeseries.value,
        });

        if (timeseries.hasCost()) {
          new DateAggregatedChart({
            renderer: this.getRenderer(index),
            mainChartsEl: chartsEl,
            chartBoxesEl: chartZoomboxesEl,
            index: index++,
            transform,
            timeseries,
            title,
            values: timeseries.cost,
            uomOverride: '$ CAD',
          });
        }
      }
    });

    const zoom = d3
      .zoom<SVGSVGElement, unknown>()
      .scaleExtent([1, 100])
      .translateExtent([
        [0, 0],
        [
          DateAggregatedChartRenderer.WIDTH(),
          DateAggregatedChartRenderer.HEIGHT(),
        ],
      ])
      .on('zoom', (e: d3.D3ZoomEvent<SVGSVGElement, never>) => {
        transform.value = e.transform;
      });
    d3.select(chartZoomboxesEl)
      .call(zoom as any)
      .call(zoom.transform as any, d3.zoomIdentity);
  }

  getRenderer(index: number) {
    if (index < this.renderers.length) {
      return this.renderers[index];
    }
    let renderer = new DateAggregatedChartRenderer();
    this.renderers.push(renderer);
    return renderer;
  }
}

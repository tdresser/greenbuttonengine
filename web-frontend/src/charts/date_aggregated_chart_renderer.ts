import * as PIXI from 'pixi.js';
import * as d3 from 'd3';

const BASE_HEIGHT = 300;

export class DateAggregatedChartRenderer {
  application: PIXI.Application<HTMLCanvasElement>;
  graphics: PIXI.Graphics;
  svg: d3.Selection<SVGGElement, undefined, null, undefined>;
  xAxisEl: d3.Selection<SVGGElement, undefined, null, undefined>;
  yAxisEl: d3.Selection<SVGGElement, undefined, null, undefined>;

  mainEl: HTMLElement;
  chartBoxEl: HTMLElement;

  titleSelection: d3.Selection<SVGTextElement, undefined, null, undefined>;
  uomSelection: d3.Selection<SVGTextElement, undefined, null, undefined>;

  static readonly MARGIN = {
    top: 50,
    bottom: 100,
    left: 100,
    right: 50,
  };

  static HEIGHT() {
    return (
      BASE_HEIGHT -
      DateAggregatedChartRenderer.MARGIN.top -
      DateAggregatedChartRenderer.MARGIN.bottom
    );
  }
  static WIDTH() {
    return (
      window.innerWidth -
      DateAggregatedChartRenderer.MARGIN.left -
      DateAggregatedChartRenderer.MARGIN.right
    );
  }

  constructor() {
    let width = DateAggregatedChartRenderer.WIDTH();
    let height = DateAggregatedChartRenderer.HEIGHT();

    this.graphics = new PIXI.Graphics();
    this.application = new PIXI.Application<HTMLCanvasElement>({
      backgroundColor: 0xcccccc,
      width,
      height,
    });
    this.application.stage.addChild(this.graphics);

    let chartSVG = d3
      .create('svg')
      .attr(
        'width',
        width +
        DateAggregatedChartRenderer.MARGIN.left +
        DateAggregatedChartRenderer.MARGIN.right,
      )
      .attr(
        'height',
        height +
        DateAggregatedChartRenderer.MARGIN.top +
        DateAggregatedChartRenderer.MARGIN.bottom,
      );

    this.svg = chartSVG
      .append('g')
      .attr(
        'transform',
        'translate(' +
        DateAggregatedChartRenderer.MARGIN.left +
        ',' +
        DateAggregatedChartRenderer.MARGIN.top +
        ')',
      );

    this.titleSelection = this.svg
      .append('text')
      .attr('class', 'title')
      .attr('x', width / 2)
      .attr('y', height + 40)
      .attr('text-anchor', 'middle');

    this.uomSelection = this.svg
      .append('text')
      .attr('class', 'title')
      .attr('x', -(DateAggregatedChartRenderer.HEIGHT() / 2))
      .attr('y', -60)
      .attr('text-anchor', 'middle')
      .attr('transform', 'rotate(-90)');

    this.xAxisEl = this.svg
      .append('g')
      .attr('transform', `translate(0,${height})`);

    this.yAxisEl = this.svg.append('g');

    this.mainEl = document.createElement('div');
    this.mainEl.appendChild(chartSVG.node()!);

    this.chartBoxEl = document.createElement('div');
    this.chartBoxEl.appendChild(this.application.view);
  }

  init(props: {
    mainChartsEl: HTMLElement;
    chartBoxesEl: HTMLElement;
    index: number;
    dates: Date[];
    values: d3.TypedArray;
    title: string;
    uom: string;
    xScale: d3.ScaleTime<number, number, never>;
    yScale: d3.ScaleLinear<number, number, never>;
  }) {
    props.mainChartsEl.append(this.mainEl);
    props.chartBoxesEl.append(this.chartBoxEl);

    (this.mainEl.style as any)['anchor-name'] = '--chart-' + props.index;
    this.mainEl.style.position = 'relative';

    (this.chartBoxEl.style as any)['position-anchor'] =
      '--chart-' + props.index;
    this.chartBoxEl.style.position = 'absolute';
    this.chartBoxEl.style.top = `calc(anchor(top) + ${DateAggregatedChartRenderer.MARGIN.top}px)`;
    this.chartBoxEl.style.left = `calc(anchor(left) + ${DateAggregatedChartRenderer.MARGIN.left}px)`;

    if (props.dates.length < 2) {
      return;
    }

    this.titleSelection.text(props.title);
    this.uomSelection.text(props.uom);

    let g = this.graphics;
    g.moveTo(props.xScale(props.dates[0]), props.yScale(props.values[0]));
    g.lineStyle({ width: 10, native: true, color: 0xff0000 });
    for (let i = 0; i < props.dates.length; ++i) {
      g.lineTo(props.xScale(props.dates[i]), props.yScale(props.values[i]));
    }
  }

  draw(
    transform: d3.ZoomTransform,
    xScale: d3.ScaleTime<number, number, never>,
    yScale: d3.ScaleLinear<number, number, never>,
  ) {
    let xAxis: d3.Axis<Date> = (
      d3.axisBottom(transform.rescaleX(xScale)) as d3.Axis<Date>
    ).tickFormat(d3.timeFormat('%Y-%m-%d'));
    let yAxis: d3.Axis<number> = d3.axisLeft(yScale) as d3.Axis<number>;

    this.xAxisEl.call(xAxis);
    this.yAxisEl.call(yAxis);

    this.graphics.transform.scale.x = transform.k;
    this.graphics.transform.position.set(transform.x, 1);
  }

  clear() {
    this.graphics.clear();
  }
}

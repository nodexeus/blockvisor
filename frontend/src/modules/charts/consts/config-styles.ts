import type { ChartDataset, ChartOptions } from 'chart.js';

export const CONFIG_DATASET_LINE_DEFAULTS: ChartDataset['data'] = {
  tension: 0,
  showLine: true,
  interpolate: true,
  borderWidth: 1,
  borderJoinStyle: 'round',
  fill: true,
  pointRadius: 0,
};

export const CONFIG_DATASET_STYLES: ChartDataset<any> = {
  line: [
    {
      ...CONFIG_DATASET_LINE_DEFAULTS,
      borderColor: 'hsl(29, 82%, 80%)',
    },
  ],
};

export const CONFIG_OPTIONS_DEFAULT: ChartOptions = {
  responsive: true,
  elements: {
    line: {
      tension: 0,
    },
  },
};

export const CONFIG_ANIMATIONS: ChartOptions = {
  animation: {
    duration: 800,
    easing: 'easeInOutQuad',
  },
  animations: {
    numbers: {
      properties: ['y', 'borderWidth', 'radius', 'tension'],
    },
  },
};

export const CONFIG_SCALES_GRID: ChartOptions<any> = {
  drawBorder: true,
  drawTicks: false,
  lineWidth: 0,
  borderWidth: 1,
  borderColor: 'rgba(248, 250, 246, 0.1)',
};

export const CONFIG_SCALES_HIDE_TICKS_TITLES = {
  title: {
    display: false,
  },
  ticks: {
    display: false,
  },
};

export const CONFIG_PLUGINS_CROSSHAIR = {
  crosshair: {
    line: {
      color: '#5F615D',
      width: 1,
    },
    zoom: { enabled: false },
    sync: { enabled: false },
  },
};

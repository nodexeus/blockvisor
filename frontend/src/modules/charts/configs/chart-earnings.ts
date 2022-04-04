import type { ChartConfiguration } from 'chart.js';
import enUS from 'date-fns/locale/en-US/index.js';
import {
  CONFIG_ANIMATIONS,
  CONFIG_OPTIONS_DEFAULT,
  CONFIG_PLUGINS_CROSSHAIR,
  CONFIG_SCALES_GRID,
  CONFIG_SCALES_HIDE_TICKS_TITLES,
} from '../consts';
import { tooltipEarnings } from '../utils';

export const CONFIG_EARNINGS: ChartConfiguration<'line'> = {
  type: 'line',
  adapters: {
    date: {
      locale: enUS,
    },
  },
  options: {
    ...CONFIG_OPTIONS_DEFAULT,
    ...CONFIG_ANIMATIONS,
    maintainAspectRatio: false,

    plugins: {
      ...CONFIG_PLUGINS_CROSSHAIR,
      legend: {
        display: false,
      },
      tooltip: {
        enabled: false,
        mode: 'interpolate',
        intersect: false,
        external: tooltipEarnings,
        callbacks: {
          title: function (a) {
            const { x, y } = a[0].element;
            return [Math.round(x).toString(), y.toFixed(2).toString()];
          },
          label: () => null,
        },
      },
    },
    scales: {
      x: {
        ...CONFIG_SCALES_HIDE_TICKS_TITLES,
        type: 'time',
        grid: CONFIG_SCALES_GRID,
      },
      y: {
        ...CONFIG_SCALES_HIDE_TICKS_TITLES,
        type: 'linear',
        beginAtZero: true,
        grid: CONFIG_SCALES_GRID,
      },
    },
  },
};

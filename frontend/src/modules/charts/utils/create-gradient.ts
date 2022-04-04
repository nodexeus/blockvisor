import type { Chart } from 'chart.js';

export const createGradient = (chart: Chart) => {
  const gradientFill = chart.ctx.createLinearGradient(
    0,
    0,
    0,
    chart.chartArea.height,
  );
  gradientFill.addColorStop(0, 'rgba(246, 203, 164, 0.17)');
  gradientFill.addColorStop(1, 'rgba(246, 203, 164, 0)');
  return gradientFill;
};

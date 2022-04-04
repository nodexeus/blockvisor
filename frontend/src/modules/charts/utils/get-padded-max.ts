import type { ScatterDataPoint } from 'chart.js';

export const getPaddedMax = (
  dataArray: ScatterDataPoint[][],
  padding = 0.5,
) => {
  const maxValue = Math.max(
    ...dataArray.map((data) => Math.max(...data.map(({ y }) => y))),
  );
  return maxValue + maxValue * padding;
};

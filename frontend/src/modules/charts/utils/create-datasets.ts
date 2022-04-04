import type { ChartDataset } from 'chart.js';
import { CONFIG_DATASET_STYLES } from '../consts';

export const createDatasets = (
  dataArray: ChartDataset[],
  type: keyof typeof CONFIG_DATASET_STYLES,
) => {
  const datasets = dataArray.map((data, index) => {
    return {
      data,
      ...CONFIG_DATASET_STYLES[type][index],
    };
  });

  return datasets;
};

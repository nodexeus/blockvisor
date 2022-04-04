import type { Chart } from 'chart.js';
import { format } from 'date-fns';

export const getOrCreateTooltip = (chart: Chart) => {
  let tooltipEl = chart.canvas.parentNode.querySelector<HTMLElement>('aside');

  if (!tooltipEl) {
    tooltipEl = document.createElement('aside');
    tooltipEl.className = 'chart__tooltip';

    const label = document.createElement('span');
    label.className = 'label t-color-text-3 t-microlabel';

    const value = document.createElement('strong');
    value.className = 'value t-color-secondary';

    tooltipEl.appendChild(value);
    tooltipEl.appendChild(label);

    chart.canvas.parentNode.appendChild(tooltipEl);
  }

  return tooltipEl;
};

export const tooltipEarnings = (context) => {
  const { chart, tooltip } = context;
  const tooltipEl = getOrCreateTooltip(chart);

  tooltipEl.style.opacity = tooltip.opacity === 0 ? '0' : '1';

  if (tooltip.body) {
    const [x = 0, y = 0] = tooltip?.title;

    const dateValue = format(parseInt(x), 'dd MMM yyyy');
    const timeValue = format(parseInt(x), 'HH:mm') + ' GMT + 1';

    const value = document.createTextNode('$ ' + y);
    const dateLabel = document.createTextNode(dateValue);
    const lineBreak = document.createElement('br');
    const timeLabel = document.createTextNode(timeValue);

    const valueRoot = tooltipEl.querySelector('.value');
    const labelRoot = tooltipEl.querySelector('.label');

    while (valueRoot.firstChild) {
      valueRoot.firstChild.remove();
    }

    while (labelRoot.firstChild) {
      labelRoot.firstChild.remove();
    }

    valueRoot.appendChild(value);
    labelRoot.appendChild(dateLabel);
    labelRoot.appendChild(lineBreak);
    labelRoot.appendChild(timeLabel);
  }

  const { offsetLeft: positionX, offsetTop: positionY } = chart.canvas;

  const isLeft = tooltip.caretX < chart.width / 2;

  if (isLeft) {
    tooltipEl.style.transform = 'translate3d(-1px,0,0)';
    tooltipEl.style.textAlign = 'left';
  } else {
    tooltipEl.style.textAlign = 'right';
    tooltipEl.style.transform = 'translate3d(calc(-100%),0,0)';
  }

  tooltipEl.style.left = positionX + tooltip.caretX + 'px';
  tooltipEl.style.top = '0';

  tooltipEl.style.top = positionY + 'px';
};

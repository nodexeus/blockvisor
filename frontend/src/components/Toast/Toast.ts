import { toast as showToast } from '@zerodevx/svelte-toast';

export const toast = {
  success: (text: string) =>
    showToast.push(text, {
      theme: {
        '--toastBackground': '#BFF589',
        '--toastColor': '#212423',
      },
    }),
  warning: (text: string) =>
    showToast.push(text, {
      theme: {
        '--toastBackground': '#EE7070',
        '--toastColor': '#212423',
      },
    }),
  message: (text: string) =>
    showToast.push(text, {
      theme: {
        '--toastBackground': '#E9AD39',
        '--toastColor': '#212423',
      },
    }),
};

// export const warning = m => toast.push(m, { theme: { ... } })

// export const failure = m => toast.push(m, { theme: { ... } })

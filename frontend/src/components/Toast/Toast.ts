import { toast as showToast } from '@zerodevx/svelte-toast';

export const toast = {
  success: (text: string) =>
    showToast.push(text, {
      theme: {
        '--toastBackground': 'hsl(143, 61%, 37%)',
        '--toastColor': 'white',
        '--toastBarBackground': 'hsl(143, 61%, 25%)',
      },
    }),
  warning: (text: string) =>
    showToast.push(text, {
      theme: {
        '--toastBackground': 'hsl(0, 79%, 69%)',
        '--toastColor': 'white',
        '--toastBarBackground': 'hsl(0, 69%, 59%)',
      },
    }),
  message: (text: string) =>
    showToast.push(text, {
      theme: {
        '--toastBackground': 'hsl(40, 80%, 57%)',
        '--toastColor': 'white',
        '--toastBarBackground': 'hsl(40, 80%, 37%)',
      },
    }),
};

// export const warning = m => toast.push(m, { theme: { ... } })

// export const failure = m => toast.push(m, { theme: { ... } })

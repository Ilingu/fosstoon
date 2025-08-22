import type { Toast } from "../types";

export let toasts = $state<Toast[]>([]);

export const pushToast = (toast: Toast) => {
  toast.id = toasts.length;
  toasts.push(toast);
  setTimeout(() => {
    let index = toasts.findIndex((t) => t.id === toast.id);
    if (index !== -1) toasts.splice(index, 1);
  }, toast.duration ?? 2500);
};

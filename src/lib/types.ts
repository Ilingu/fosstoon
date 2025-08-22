export type Toast = {
  content: string;
  alert: "none" | "success" | "info" | "warning" | "error";
  duration?: number;
  id?: number;
};

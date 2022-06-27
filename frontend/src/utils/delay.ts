export const delay = async (delay: number) =>
  new Promise((resolve, _reject) => setTimeout(resolve, delay));

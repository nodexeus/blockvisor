export const isObjectEmpty = (o: { [k: string]: unknown }) =>
  !Object.keys(o).length;

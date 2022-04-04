import type { UiNode } from '@ory/kratos-client';

export const reorderFields = (nodes: UiNode[], order: string[]) => {
  const nodesOrdered = [];

  for (let i = 0; i < nodes?.length; i++) {
    const currentNode = nodes[i];
    const index = order?.indexOf(currentNode.attributes?.name) ?? -1;
    if (index > -1) {
      nodesOrdered[index] = currentNode;
    } else {
      nodesOrdered.push(currentNode);
    }
  }

  return nodesOrdered.filter((node) => !!node);
};

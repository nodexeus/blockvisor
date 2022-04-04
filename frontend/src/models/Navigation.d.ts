interface NavNode {
  isParent?: boolean;
  href: string;
  title: string;
  children?: NavNode[];
}

type SubMenu = 'nodes' | 'hosts' | '';

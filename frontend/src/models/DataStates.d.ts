type StatusState = 'default' | 'active' | 'inactive' | 'error';

type NodeState =
  | 'staking'
  | 'syncing'
  | 'updating'
  | 'consensus'
  | 'unstaked'
  | 'disabled';

type HostState = 'pending' | 'normal' | 'loaded' | 'issue';

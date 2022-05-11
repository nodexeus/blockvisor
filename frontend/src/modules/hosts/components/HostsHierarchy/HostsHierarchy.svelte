<script lang="ts">
  import HierarchyList from 'components/HierarchyList/HierarchyList.svelte';

  import { user } from 'modules/authentication/store';
  import GroupEdit from 'modules/forms/components/GroupEdit/GroupEdit.svelte';
  import NodeGroupAdd from 'modules/nodes/components/NodeGroupAdd/NodeGroupAdd.svelte';
  import { onMount } from 'svelte';
  import { fetchAllHosts, hosts } from 'modules/hosts/store/hostsStore';

  let isAddingNewGroup = false;
  let editingId;

  const handleEdit = (e) => (editingId = e.target.value);
  const handleEditConfirm = () => (editingId = null);
  const handleAddConfirm = () => (isAddingNewGroup = false);
  const handleAddGroup = () => (isAddingNewGroup = true);

  onMount(() => {
    fetchAllHosts($user);
  });
</script>

<HierarchyList
  on:click
  handleConfirm={handleEditConfirm}
  {handleEdit}
  {editingId}
  nodes={$hosts}
>
  {#if isAddingNewGroup}
    <li>
      <GroupEdit handleConfirm={handleAddConfirm} />
    </li>
  {/if}

  <NodeGroupAdd
    disabled={isAddingNewGroup}
    on:click={handleAddGroup}
    slot="action"
  />
</HierarchyList>

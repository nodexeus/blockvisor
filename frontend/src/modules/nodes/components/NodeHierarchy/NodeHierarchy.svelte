<script lang="ts">
  import HierarchyList from 'components/HierarchyList/HierarchyList.svelte';
  import GroupEdit from 'modules/forms/components/GroupEdit/GroupEdit.svelte';
  import NodeGroupAdd from '../NodeGroupAdd/NodeGroupAdd.svelte';
  export let nodes;

  let isAddingNewGroup = false;
  let editingId = null;

  const handleEdit = (e) => (editingId = e.target.value);
  const handleEditConfirm = () => (editingId = null);
  const handleAddConfirm = () => (isAddingNewGroup = false);
  const handleAddGroup = () => (isAddingNewGroup = true);
</script>

<HierarchyList
  on:click
  handleConfirm={handleEditConfirm}
  {handleEdit}
  {editingId}
  {nodes}
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

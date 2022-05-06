<script lang="ts">
  import axios from 'axios';
  import { nodes , fetchAllNodes} from 'modules/nodes/store/nodesStore';
  import HierarchyList from 'components/HierarchyList/HierarchyList.svelte';
  import { ROUTES } from 'consts/routes';
  import { NODES } from 'modules/authentication/const';
  import GroupEdit from 'modules/forms/components/GroupEdit/GroupEdit.svelte';
  import { onMount } from 'svelte';
  import NodeGroupAdd from '../NodeGroupAdd/NodeGroupAdd.svelte';
  import { app } from 'modules/app/store';
  import { claim_svg_element } from 'svelte/internal';


  let isAddingNewGroup = false;
  let editingId = null;

  const handleEdit = (e) => (editingId = e.target.value);
  const handleEditConfirm = () => (editingId = null);
  const handleAddConfirm = () => (isAddingNewGroup = false);
  const handleAddGroup = () => (isAddingNewGroup = true);


  onMount(() => {
    fetchAllNodes();
  });
</script>

<HierarchyList
  on:click
  handleConfirm={handleEditConfirm}
  {handleEdit}
  {editingId}
  nodes={$nodes}
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

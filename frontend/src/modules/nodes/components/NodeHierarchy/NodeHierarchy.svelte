<script lang="ts">
  import axios from 'axios';
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

  let all_nodes = [
    {
      title: 'All Nodes',
      href: ROUTES.NODES,
      children: [],
    },
  ];

  const groupBy = (array, key) => {
    return array.reduce((result, currentValue) => {
      (result[currentValue[key]] = result[currentValue[key]] || []).push(
        currentValue,
      );
      return result;
    }, {});
  };

  onMount(async () => {
    const token =
      'eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzUxMiJ9.eyJzdWIiOiI4OWI3MzgyMi04ODM3LTQ5NTAtOTA4Yy0zZTNiM2E4MjJlMzQiLCJyb2xlIjoiYWRtaW4iLCJleHAiOjE2NTE4NDA0MTV9.i_TPUQ7kN8mXJ5i793q3BcvcbYP_n_oNWU-OHjujzl4I0oxEIDbsNEHqcnJVm6sPZTgOV3SUHM-TjAqcWMNsdw';

    const res = await axios.get(NODES, {
      headers: { Authorization: `Bearer ${token}` },
    });

    const filteredData = res.data.slice(0, 100);

    const groupedData = Object.entries(groupBy(filteredData, 'user_id'));

    all_nodes[0].children = groupedData.map((item) => {
      return {
        title: item[0],
        href: ROUTES.NODE_GROUP(item[0]),
        id: 'host-id1',
      };
    });

    app.setNodes(all_nodes);
  });
</script>

<HierarchyList
  on:click
  handleConfirm={handleEditConfirm}
  {handleEdit}
  {editingId}
  nodes={all_nodes}
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

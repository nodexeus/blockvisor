<script lang="ts">
  import ActionTitleHeader from 'components/ActionTitleHeader/ActionTitleHeader.svelte';
  import { pageTransition } from 'consts/animations';
  import { fade } from 'svelte/transition';
  import EditUser from 'modules/admin-console/components/EditUser/EditUser.svelte';
</script>

<section in:fade|local={pageTransition}>
  <ActionTitleHeader className="container--pull-back">
    <h2 class="t-large" slot="title">Edit User</h2>
  </ActionTitleHeader>
</section>

<section in:fade|local={pageTransition} class="edit container--medium-large">
  <EditUser />
</section>

<style>
</style>

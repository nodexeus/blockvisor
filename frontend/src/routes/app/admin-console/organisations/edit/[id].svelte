<script lang="ts">
  import ActionTitleHeader from 'components/ActionTitleHeader/ActionTitleHeader.svelte';
  import { pageTransition } from 'consts/animations';
  import { ROUTES } from 'consts/routes';
  import { app } from 'modules/app/store';
  import EditOrganization from 'modules/organisation/components/EditOrganization.svelte';
  import {
    getOrganisationById,
    selectedOrganization,
  } from 'modules/organisation/store/organisationStore';
  import { onMount } from 'svelte';
  import { fade } from 'svelte/transition';
  import { page } from '$app/stores';
  import LoadingSpinner from 'components/Spinner/LoadingSpinner.svelte';

  const id = $page.params.id;

  onMount(() => {
    getOrganisationById(id);

    setTimeout(() => {
      app.setBreadcrumbs([
        {
          title: 'Admin Console',
          url: ROUTES.ADMIN_CONSOLE_DASHBOARD,
        },
        {
          title: 'Organisations',
          url: ROUTES.ADMIN_CONSOLE_ORGANISATIONS,
        },
        {
          title: 'Edit Organisation',
          url: '',
        },
      ]);
    }, 200);
  });
</script>

<section in:fade|local={pageTransition}>
  <ActionTitleHeader className="container--pull-back">
    <h2 class="t-large" slot="title">Edit Organisation</h2>
  </ActionTitleHeader>
</section>

<section in:fade|local={pageTransition} class="edit container--medium-large">
  {#if !$selectedOrganization}
    <LoadingSpinner size="medium" id="js-form-submit" />
  {:else}
    <EditOrganization />
  {/if}
</section>

<style>
</style>

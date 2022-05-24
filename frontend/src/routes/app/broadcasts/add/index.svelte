<script context="module" lang="ts">
  export async function load({ url }) {
    const id = url.searchParams.get('id');

    if (id) {
      const thisBroadcast: Broadcast = await getBroadcastById(id);

      return {
        props: {
          broadcast: thisBroadcast,
        },
      };
    }

    return {};
  }
</script>

<script>
  import SectionDescription from 'components/SectionDescription/SectionDescription.svelte';
  import { fadeDefault } from 'consts/animations';
  import { user } from 'modules/authentication/store/auth';
  import AddBroadcast from 'modules/broadcasts/components/AddBroadcast/AddBroadcast.svelte';
  import BroadcastFaq from 'modules/broadcasts/components/AddBroadcast/BroadcastFaq.svelte';
  import {
    getBroadcastById,
    getOrganisationId,
  } from 'modules/broadcasts/store/broadcastStore';
  import { onMount } from 'svelte';
  import { fade } from 'svelte/transition';

  export let broadcast;

  onMount(() => {
    getOrganisationId($user.id);
  });
</script>

<section in:fade={fadeDefault} class="grid broadcast-add">
  <div class="container-medium broadcast-add__wrapper">
    <h2 class="t-xxlarge-fluid">
      {#if broadcast}
        Edit {broadcast.name}
      {:else}
        Add a Broadcast
      {/if}
    </h2>

    <SectionDescription>
      Broadcast allows you to be instantly notified of any transaction matching
      your criteria below. Once a new block is produced on the chain, our
      broadcast service will inspect it for any transactions matching your
      filters below and will instantly send the json (and oracle price) to your
      callback.
    </SectionDescription>

    <AddBroadcast initial={broadcast} />
  </div>
  <div class="divider" />
  <div class="container-medium faq">
    <BroadcastFaq />
  </div>
</section>

<style>
  .broadcast-add {
    grid-row-gap: 0;
  }

  .broadcast-add__wrapper {
    margin-top: 40px;
    margin-bottom: 80px;
    grid-column: 1 / span 12;

    @media (--screen-medium-large) {
      grid-column: 2 / 6;
    }
  }

  .divider {
    border-left: 1px solid var(--color-text-5-o10);
    grid-column: 7/7;
  }

  .faq {
    padding-top: 40px;
    padding-bottom: 80px;
    grid-column: 1 / span 12;

    @media (--screen-medium-large) {
      grid-column: 8/ 12;
    }
  }
</style>

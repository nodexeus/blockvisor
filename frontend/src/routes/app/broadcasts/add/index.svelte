<script>
  import { fade } from 'svelte/transition';
  import AddBroadcast from 'modules/broadcasts/components/AddBroadcast/AddBroadcast.svelte';
  import { fadeDefault } from 'consts/animations';
  import SectionDescription from 'components/SectionDescription/SectionDescription.svelte';
</script>

<section in:fade={fadeDefault} class="grid broadcast-add">
  <div class="container--medium broadcast-add__wrapper">
    <h2 class="t-xxlarge-fluid">Add a Broadcast</h2>

    <SectionDescription>
      Broadcast allows you to be instantly notified of any transaction matching
      your criteria below. Once a new block is produced on the chain, our
      broadcast service will inspect it for any transactions matching your
      filters below and will instantly send the json (and oracle price) to your
      callback.
    </SectionDescription>

    <AddBroadcast />
  </div>
</section>

<style>
  .broadcast-add {
    margin-top: 40px;
    margin-bottom: 80px;
  }
  .broadcast-add__wrapper {
    grid-column: 1 / span 12;

    @media (--screen-medium-large) {
      grid-column: 2 / span 12;
    }
  }
</style>

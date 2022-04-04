<script>
  import { fade } from 'svelte/transition';
  import AddBroadcast from 'modules/broadcasts/components/AddBroadcast/AddBroadcast.svelte';
  import { fadeDefault } from 'consts/animations';
</script>

<section in:fade={fadeDefault} class="grid broadcast-add">
  <div class="container--small broadcast-add__wrapper">
    <h2 class="t-xxlarge-fluid">Add a Broadcast</h2>

    <AddBroadcast />
  </div>
</section>

<style>
  .broadcast-add {
    margin-top: 40px;

    &__wrapper {
      grid-column: 1 / span 12;

      @media (--screen-medium-large) {
        grid-column: 2 / span 12;
      }
    }
  }
</style>

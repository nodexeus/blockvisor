<script>
  import { fade } from 'svelte/transition';
  import { pageTransition } from 'consts/animations';
</script>

<section in:fade|local={pageTransition}>Admin console dashboard</section>

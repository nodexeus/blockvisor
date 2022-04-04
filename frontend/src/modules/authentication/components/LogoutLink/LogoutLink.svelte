<script lang="ts">
  import { getContext } from 'svelte';
  const logoutUrl: string = getContext('logoutUrl');
</script>

<a class="link link--primary" href={logoutUrl} {...$$restProps}><slot /></a>

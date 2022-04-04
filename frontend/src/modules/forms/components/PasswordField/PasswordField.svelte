<script lang="ts">
  import Input from 'modules/forms/components/Input/Input.svelte';
  export let type = 'password';
</script>

<Input {...$$props} {type}>
  <slot slot="utilRight" name="utilRight" />
  <slot slot="label" name="label" />
  <slot slot="hints" name="hints" />
</Input>

<script lang="ts">
  import Button from 'components/Button/Button.svelte';
  import TextWithAction from 'modules/dashboard/components/TextWithAction/TextWithAction.svelte';

  export let setStep;

  const handleClick = (e: MouseEvent) => {
    setStep(parseInt(e.target.dataset.step));
  };
</script>

<TextWithAction>
  <svelte:fragment slot="title">Step 1: Download blockvisor</svelte:fragment>
  <Button slot="action" asLink size="small" style="outline">Download</Button>
  <svelte:fragment>
    Maecenas eget lorem auctor, posuere ante luctus, tincidunt nulla. Praesent
    iaculis purus mattis bibendum sollicitudin. Morbi tristique tellus semper,
    scelerisque massa.
  </svelte:fragment>
</TextWithAction>

<TextWithAction>
  <svelte:fragment slot="title">Step 2: Set up your keys</svelte:fragment>
  <svelte:fragment slot="action">EQ2WBtEt50</svelte:fragment>
  <svelte:fragment>
    Maecenas eget lorem auctor, posuere ante luctus, tincidunt nulla. Praesent
    iaculis purus mattis bibendum sollicitudin. Morbi tristique tellus semper,
    scelerisque massa.
  </svelte:fragment>
</TextWithAction>

<Button
  on:click={handleClick}
  data-step="3"
  size="medium"
  style="secondary"
  type="button">Finish</Button
>
<Button
  on:click={handleClick}
  data-step="1"
  size="medium"
  style="ghost"
  type="button">Go Back</Button
>

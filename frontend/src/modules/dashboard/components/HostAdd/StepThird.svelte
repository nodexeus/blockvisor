<script lang="ts">
  import Button from 'components/Button/Button.svelte';
  import Pill from 'components/Pill/Pill.svelte';
  import HostFormData from 'modules/dashboard/components/HostFormData/HostFormData.svelte';

  export let setStep;

  const handleClick = (e) => {
    setStep(parseInt(e.target.dataset.step));
  };
</script>

<ul class="u-list-reset step-list ">
  <HostFormData>
    <svelte:fragment slot="label">Host name</svelte:fragment>
    <svelte:fragment slot="value">HostIguana</svelte:fragment>
  </HostFormData>
  <HostFormData>
    <svelte:fragment slot="label">Tags</svelte:fragment>
    <ul class="u-list-reset" slot="value">
      <li>
        <Pill transition={{ duration: 0, delay: 0 }} removable={false}>eth</Pill
        >
        <Pill transition={{ duration: 0, delay: 0 }} removable={false}
          >validator</Pill
        >
        <Pill transition={{ duration: 0, delay: 0 }} removable={false}
          >americas</Pill
        >
      </li>
    </ul>
  </HostFormData>
  <HostFormData>
    <svelte:fragment slot="label">Host location</svelte:fragment>
    <svelte:fragment slot="value">Mexico, Central America</svelte:fragment>
  </HostFormData>
  <HostFormData>
    <svelte:fragment slot="label">Host key</svelte:fragment>
    <svelte:fragment slot="value">EQ2WBtEt50</svelte:fragment>
  </HostFormData>
</ul>

<Button size="medium" style="secondary" type="button">Finish</Button>
<Button
  on:click={handleClick}
  data-step="2"
  size="medium"
  style="ghost"
  type="button">Go Back</Button
>

<style>
  .step-list {
    margin-top: 56px;
    margin-bottom: 40px;
  }
</style>

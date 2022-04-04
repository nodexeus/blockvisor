<script lang="ts">
  import StepLabel from 'components/StepLabel/StepLabel.svelte';
  import { fade } from 'svelte/transition';
  import StepList from 'components/StepList/StepList.svelte';
  import { MetaTags } from 'svelte-meta-tags';

  import { app } from 'modules/app/store';
  import HostAdd from 'modules/dashboard/components/HostAdd/HostAdd.svelte';
  import { onMount } from 'svelte';

  let currentStep = 1;

  const setStep = (step: number) => {
    currentStep = step;
  };

  onMount(() => {
    app.setBreadcrumbs([
      {
        title: 'Dashboard',
        url: '/dashboard',
      },
      {
        title: 'Add a host',
        url: '/host/add',
      },
    ]);
  });
</script>

<MetaTags title="Add new host | BlockVisor" />

<div class="add-host container--large grid grid-spacing">
  <aside class="add-host__sidebar">
    <StepList {currentStep}>
      <li in:fade={{ duration: 250 }}>
        <StepLabel {setStep} {currentStep} step={1}>Add a new host</StepLabel>
      </li>
      <li in:fade={{ duration: 250, delay: 220 }}>
        <StepLabel {setStep} {currentStep} step={2}>Add a new host</StepLabel>
      </li>
      <li in:fade={{ duration: 250, delay: 440 }}>
        <StepLabel {setStep} {currentStep} step={3}>Add a new host</StepLabel>
      </li>
    </StepList>
  </aside>
  <HostAdd {setStep} {currentStep} />
</div>

<style>
  .add-host {
    align-items: flex-start;

    @media (--screen-large) {
      height: 100%;
    }

    &__sidebar {
      grid-column: span 12;
      padding-top: 40px;
      text-align: center;
      overflow-x: hidden;

      @media (--screen-large) {
        text-align: left;
        grid-column: span 3;
        height: 100%;
        border-right: 1px solid theme(--color-text-5-o10);
      }
    }
  }
</style>

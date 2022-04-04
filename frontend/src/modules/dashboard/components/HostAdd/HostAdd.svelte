<script lang="ts">
  import { onMount } from 'svelte';

  import { useForm } from 'svelte-use-form';

  import Layout from './Layout.svelte';

  import StepFirst from './StepFirst.svelte';
  import StepSecond from './StepSecond.svelte';
  import StepThird from './StepThird.svelte';

  const components = [StepFirst, StepSecond, StepThird];
  const layoutProps = [
    { title: 'Add a host', size: 'medium' },
    { title: 'Setup your host', size: 'large' },
    { title: 'Final checkup', size: 'large' },
  ];

  let activeComponent = null;

  const form = useForm();

  export let currentStep = 1;
  export let setStep;

  const updateViewportComponent = () => {
    activeComponent = components[currentStep - 1];
  };

  onMount(updateViewportComponent);
</script>

{#if activeComponent === components[currentStep - 1]}
  <Layout
    on:outroend={updateViewportComponent}
    {...layoutProps[currentStep - 1]}
  >
    <svelte:component this={activeComponent} {setStep} {form} />
  </Layout>
{/if}

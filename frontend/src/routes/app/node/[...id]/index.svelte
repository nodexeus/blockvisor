<script lang="ts">
  import { page } from '$app/stores';
  import MinimalLineGraph from 'modules/charts/components/MinimalLineGraph/MinimalLineGraph.svelte';
  import { CONFIG_EARNINGS } from 'modules/charts/configs/chart-earnings';
  import DetailsTable from 'modules/nodes/components/DetailsTable/DetailsTable.svelte';
  import DetailsLayout from 'modules/nodes/components/DetailsLayout/DetailsLayout.svelte';
  import DetailsHeader from 'modules/nodes/components/DetailsHeader/DetailsHeader.svelte';
  import { onMount } from 'svelte';
  import { app } from 'modules/app/store';
  import { useForm } from 'svelte-use-form';
  import MapSection from 'modules/nodes/components/MapSection/MapSection.svelte';
  import BackButton from 'modules/app/components/BackButton/BackButton.svelte';
  import { ROUTES } from 'consts/routes';
  import { fetchValidatorById, selectedValidator, isLoading } from 'modules/nodes/store/nodesStore';
import LoadingSpinner from 'components/Spinner/LoadingSpinner.svelte';

  const id = $page.params.id;

  $: {
    fetchValidatorById($page.params.id);
  }

  onMount(() => {
    app.setBreadcrumbs([
      {
        title: 'Nodes',
        url: ROUTES.NODES,
      },
      {
        title: 'YellowBeaver',
        url: '',
      },
    ]);
  });

  var getDaysArray = function (s, e) {
    for (var a = [], d = new Date(s); d <= e; d.setDate(d.getDate() + 1)) {
      const randomDate = new Date(d);
      a.push({
        x: randomDate,
        y: Math.floor(Math.random() * (100 - 60 + 1)) + 60,
      });
    }
    return a;
  };

  const PLACEHOLDER_DATA = getDaysArray(
    new Date('2022-06-01'),
    new Date('2022-07-01'),
  );

  const form = useForm();
</script>

{#if $isLoading}

  <LoadingSpinner id='js-spinner' size="page" />

{:else}
<DetailsLayout>
<BackButton slot="nav" />

<DetailsHeader data={$selectedValidator} {form} state="consensus" {id} />

<MinimalLineGraph
  height="200"
  config={CONFIG_EARNINGS}
  data={[PLACEHOLDER_DATA]}
>
  <svelte:fragment slot="label">Node earnings (USD)</svelte:fragment>
</MinimalLineGraph>

<DetailsTable data={$selectedValidator} />

<MapSection />

</DetailsLayout>
{/if}

<script>
  import anime from 'animejs';

  import Button from 'components/Button/Button.svelte';

  import PlanCard from 'components/PlanCard/PlanCard.svelte';
  import { onMount } from 'svelte';
  import SubscriptionToggle from '../SubscriptionToggle/SubscriptionToggle.svelte';

  let activeSubscription = 'yearly';

  const handleChangeSubscription = (e) => {
    activeSubscription = e.currentTarget.value;
  };

  onMount(() => {
    anime({
      targets: '#js-subscription-plans > li',
      opacity: [0, 1],
      translateY: [8, 0],
      easing: 'easeInOutQuad',
      delay: anime.stagger(300),
      duration: 400,
    });
  });
</script>

<section class="grid subscription-plans container--large">
  <header class="subscription-plans__header">
    <h2 class="subscription-plans__title t-xlarge-fluid">
      Start Your Broadcast Subscription!
    </h2>

    <div class="subscription-plans__controls">
      <SubscriptionToggle
        value="yearly"
        on:click={handleChangeSubscription}
        isActive={activeSubscription === 'yearly'}>Yearly</SubscriptionToggle
      >
      <SubscriptionToggle
        value="monthly"
        on:click={handleChangeSubscription}
        isActive={activeSubscription === 'monthly'}>Monthly</SubscriptionToggle
      >
    </div>
  </header>

  <ul id="js-subscription-plans" class="u-list-reset subscription-plans__list">
    <li>
      <PlanCard plan={['Feature 1', 'Feature 2', 'Feature 3']}>
        <svelte:fragment slot="title">Basic Plan</svelte:fragment>
        <svelte:fragment slot="fee">$99,00</svelte:fragment>
        <svelte:fragment slot="description"
          >per Month<br /> paid anually</svelte:fragment
        >
        <svelte:fragment slot="note">
          You save 12$/month by choosing the annual plan.
        </svelte:fragment>
        <Button display="block" size="medium" slot="action" style="outline"
          >Select Plan</Button
        >
      </PlanCard>
    </li>
    <li>
      <PlanCard highlight plan={['Feature 1', 'Feature 2', 'Feature 3']}>
        <svelte:fragment slot="note">
          You save 12$/month by choosing the annual plan.
        </svelte:fragment>
        <svelte:fragment slot="title">Basic Plan</svelte:fragment>
        <span slot="featured" class="t-uppercase t-label t-color-primary"
          >Best value</span
        >
        <svelte:fragment slot="fee">$199,00</svelte:fragment>
        <svelte:fragment slot="description"
          >per Month<br /> paid anually</svelte:fragment
        >
        <Button display="block" size="medium" slot="action" style="secondary"
          >Select Plan</Button
        >
      </PlanCard>
    </li>
    <li>
      <PlanCard plan={['Feature 1', 'Feature 2', 'Feature 3']}>
        <svelte:fragment slot="note">
          You save 12$/month by choosing the annual plan.
        </svelte:fragment>
        <svelte:fragment slot="title">Basic Plan</svelte:fragment>
        <svelte:fragment slot="fee">$299,00</svelte:fragment>
        <svelte:fragment slot="description"
          >per Month<br /> paid anually</svelte:fragment
        >
        <Button display="block" size="medium" slot="action" style="outline"
          >Select Plan</Button
        >
      </PlanCard>
    </li>
  </ul>
</section>

<style>
  .subscription-plans {
    grid-row-gap: 0;

    @media (--screen-medium-large) {
      margin-top: 96px;
    }

    &__header {
      display: flex;
      justify-content: space-between;
      flex-wrap: wrap;
      gap: 20px 108px;
      grid-column: span 12;
      margin-bottom: 40px;

      @media (--screen-large) {
        margin-bottom: 80px;
        grid-column: 3 / 12;
      }
    }

    &__list {
      gap: 28px;
      overflow-x: scroll;
      overflow-y: hidden;
      grid-column: span 12;
      display: flex;
      flex-direction: column;

      @media (--screen-medium-large) {
        flex-direction: row;
      }

      @media (--screen-large) {
        grid-column: 3 / 12;
      }

      > li {
        flex: 1 0 296px;
        opacity: 0;
      }
    }

    &__title {
      max-width: 404px;
    }

    &__controls {
      display: flex;
      gap: 12px;
      align-items: flex-start;
    }
  }
</style>

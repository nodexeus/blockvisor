<script lang="ts">
  import FaqItem from './FaqItem.svelte';
</script>

<section class="faq">
  <h2 class="t-large">Need help?</h2>

  <div class="faq__list">
    <FaqItem
      title="Question title about broadcast feature"
      content="Lorem ipsum dolor sit amet, consectetur adipiscing elit. Proin ultrices diam vitae hendrerit semper. Pellentesque non velit tincidunt, porttitor risus sed, elementum quam."
    />
    <FaqItem
      title="Question title about broadcast feature"
      content="Lorem ipsum dolor sit amet, consectetur adipiscing elit. Proin ultrices diam vitae hendrerit semper. Pellentesque non velit tincidunt, porttitor risus sed, elementum quam."
    />
    <FaqItem
      title="Question title about broadcast feature"
      content="Lorem ipsum dolor sit amet, consectetur adipiscing elit. Proin ultrices diam vitae hendrerit semper. Pellentesque non velit tincidunt, porttitor risus sed, elementum quam."
    />
  </div>
</section>

<style>
  .faq__list {
    margin-top: 60px;
  }
</style>

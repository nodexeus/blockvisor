<script lang="ts">
  import Button from 'components/Button/Button.svelte';
  import Input from 'modules/forms/components/Input/Input.svelte';
  import TagsField from 'modules/forms/components/TagsField/TagsField.svelte';
  import { required, Hint } from 'svelte-use-form';

  export let form;
  export let setStep;
</script>

<form
  use:form
  on:submit={(e) => {
    e.preventDefault();
    setStep(2);
  }}
>
  <ul class="u-list-reset step__inputs">
    <li class="step__input-item">
      <Input
        name="hostname"
        size="large"
        value={$form?.hostname?.value}
        field={$form?.hostname}
        validate={[required]}
        placeholder="Enter your host name"
        required
      >
        <svelte:fragment slot="label">Host Name</svelte:fragment>
        <svelte:fragment slot="hints">
          <Hint on="required">This is a mandatory field</Hint>
        </svelte:fragment>
      </Input>
    </li>

    <li class="step__input-item">
      <TagsField
        value={$form?.tags?.value}
        size="large"
        placeholder="Add tags"
        field={$form?.tags}
        name="tags"
      >
        <svelte:fragment slot="label">Tags</svelte:fragment>
      </TagsField>
    </li>
  </ul>

  <Button size="medium" style="secondary" type="submit">Next step</Button>
</form>

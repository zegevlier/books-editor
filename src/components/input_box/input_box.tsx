import { component$, useStylesScoped$ } from "@builder.io/qwik";
import { Props } from "~/routes";
import styles from "./input_box.css?inline";

export default component$((props: { state: Props }) => {
  useStylesScoped$(styles);

  return (
    <div class="input-box">
      <textarea
        value={props.state.input}
        onInput$={(e) => {
          props.state.input = (e.target as HTMLTextAreaElement).value || "";
        }}
      />
    </div>
  );
});

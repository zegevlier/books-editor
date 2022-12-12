import { component$, useStore } from "@builder.io/qwik";
import type { DocumentHead } from "@builder.io/qwik-city";
import Inputbox from "~/components/input_box/input_box";
import Bookviewer from "~/components/book_viewer/book_viewer";

export interface Props {
  input: string;
}

export default component$(() => {
  const defaultInput = `Minecraft book editor!
&cS&6u&ep&ap&9o&br&5t&cs &ec&ao&9l&bo&5u&cr&6s&e!

&r&lAnd bold
&r&oAnd italics
&rThough no underline and strikethrough yet.
Currently pixel-perfect to GUI scale 1, but looks wonky when scaled up. Will change to GUI scale 2!`;
  const state = useStore<Props>({
    input: defaultInput,
  });


  return (
    <div class="outer">
      <div>
        <h1 class="rainbow">Book editor!</h1>
      </div>
      <div id="main">
        <div class="split left">
          <Inputbox state={state} />
        </div>
        <div class="split right">
          <Bookviewer input={state.input} />
        </div>
      </div>
    </div>
  );
});

export const head: DocumentHead = {
  title: "Book editor!",
};

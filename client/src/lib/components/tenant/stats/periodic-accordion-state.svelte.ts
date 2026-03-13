
class _AccordionState {
    // element: Element | null = $state(null);

    element: HTMLElement | null | undefined = $state(null);
    lastValue = $state<string | undefined>(undefined);
    value = $derived(this.deriveInitial());

    deriveInitial() {
        const triggerElement = this.element?.getElementsByTagName("div")[0] ?? undefined;
        $inspect(triggerElement);

        if (triggerElement) {
            triggerElement.getAttribute('data-state')
            return undefined
        }
    }

}


export const AccordionState = new _AccordionState();

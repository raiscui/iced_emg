/*
 * @Author: Rais
 * @Date: 2021-06-26 16:46:38
 * @LastEditTime: 2021-06-26 23:44:07
 * @LastEditors: Rais
 * @Description:
 */

export function observeSize(element, send_msg_resized) {
    const resizeObserver = new ResizeObserver(entries => {
        let entry = entries[0];
        // for (let entry of entries) {
        // const boxEl = entry.target;
        const dimensions = entry.contentRect;
        // boxEl.textContent = `${dimensions.width} x ${dimensions.height}`;
        // }
        send_msg_resized(dimensions.width, dimensions.height);
    });
    resizeObserver.observe(element, send_msg_resized);
}

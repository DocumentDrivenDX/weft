const tabs = [...document.querySelectorAll('[role="tab"]')];
const panel = document.querySelector('#mapping');
const mappings = {
 truss: ['Objects, keys, edges.', 'Lower logical fields into Truss’s catalog and declared property homes, including JSONB, typed scalar and compound storage.'],
 ashlar: ['A publication-aware warehouse.', 'Lower logical fields into Ashlar’s Delta layout, with explicit table versions, typed columns, presence and JSON property homes.']
};
function select(tab) {
 for (const button of tabs) { button.setAttribute('aria-selected', String(button === tab)); button.tabIndex = button === tab ? 0 : -1; }
 panel.setAttribute('aria-labelledby', tab.id);
 panel.querySelector('.mapping-title').textContent = mappings[tab.dataset.target][0];
 panel.querySelector('.mapping-copy').textContent = mappings[tab.dataset.target][1];
}
for (const tab of tabs) {
 tab.addEventListener('click', () => select(tab));
 tab.addEventListener('keydown', event => {
  const index = tabs.indexOf(tab);
  const next = {ArrowRight: (index + 1) % tabs.length, ArrowLeft: (index + tabs.length - 1) % tabs.length, Home: 0, End: tabs.length - 1}[event.key];
  if (next !== undefined) {event.preventDefault(); select(tabs[next]); tabs[next].focus();}
 });
}
const links = [...document.querySelectorAll('nav a[href^="#"]')];
if ('IntersectionObserver' in window) {
 const observer = new IntersectionObserver(entries => {
  for (const entry of entries) if (entry.isIntersecting) for (const link of links) {
   if (link.hash === '#' + entry.target.id) link.setAttribute('aria-current', 'location'); else link.removeAttribute('aria-current');
  }
 }, {rootMargin: '-10% 0px -65% 0px'});
 for (const link of links) observer.observe(document.querySelector(link.hash));
}

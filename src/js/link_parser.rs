pub const LINK_PARSER_JS: &str = r#"
const links = document.querySelectorAll('a');
links.forEach((link, i) => {
    const span = document.createElement('span');
    span.textContent = '●'; // minimal visual indicator
    span.style.position = 'absolute';
    span.style.background = 'yellow';
    span.style.color = 'black';
    span.style.fontSize = '12px';
    span.style.zIndex = '9999';
    const rect = link.getBoundingClientRect();
    span.style.left = (rect.left + window.scrollX) + 'px';
    span.style.top = (rect.top + window.scrollY) + 'px';
    document.body.appendChild(span);
});
"#;

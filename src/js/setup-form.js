// The setup a person fills in before an install.
//
// Built with DOM nodes rather than markup: every label, option and default
// here comes from an upstream app definition, and interpolating that into HTML
// would make an app's own text able to write the page. Nothing is stored — the
// answers live in the form until the moment they are sent, so they cannot
// reach saved UI state, a diagnostic report or a log.
//
// Validation is the backend's job. What happens here is only enough to avoid a
// round trip for an empty required field; the rules themselves stay in Rust,
// where the regexes are compiled with bounds.

const CONTAINER = 'setup-fields';

function labelled(field, control) {
  const wrapper = document.createElement('label');
  wrapper.className = 'setup-field';
  wrapper.htmlFor = control.id;
  const name = document.createElement('span');
  name.className = 'setup-label';
  name.textContent = field.label;
  if (field.required) {
    const mark = document.createElement('span');
    mark.className = 'setup-required';
    mark.textContent = ' (required)';
    name.append(mark);
  }
  wrapper.append(name, control);
  return wrapper;
}

// What a blank answer will do, said plainly rather than left to be guessed.
function hintFor(field) {
  if (field.required) return '';
  if (!field.has_default) return 'Optional.';
  // A sensitive default is deliberately not sent to the interface, so it can
  // be described but not shown.
  return field.sensitive
    ? 'Leave blank to use the app’s own default.'
    : `Leave blank to use ${field.default}.`;
}

function controlFor(field) {
  const id = `setup-${field.key.toLowerCase().replaceAll('_', '-')}`;
  if (field.control === 'choice') {
    const select = document.createElement('select');
    select.id = id;
    if (!field.required) {
      const blank = document.createElement('option');
      blank.value = '';
      blank.textContent = field.has_default ? `Default (${field.default})` : 'Not set';
      select.append(blank);
    }
    for (const option of field.options) {
      const item = document.createElement('option');
      item.value = option;
      item.textContent = option;
      select.append(item);
    }
    if (field.has_default && field.default !== undefined) select.value = field.default;
    return select;
  }
  const input = document.createElement('input');
  input.id = id;
  if (field.control === 'boolean') {
    input.type = 'checkbox';
    input.checked = field.default === 'true';
    return input;
  }
  if (field.control === 'number') {
    input.type = 'number';
    input.inputMode = 'numeric';
    if (field.min !== undefined) input.min = String(field.min);
    if (field.max !== undefined) input.max = String(field.max);
  } else if (field.sensitive) {
    input.type = 'password';
    // Never offered to a password manager: this is the app's credential, not
    // the person's, and it is generated or chosen once.
    input.autocomplete = 'off';
    input.spellcheck = false;
  } else {
    input.type = 'text';
    input.autocomplete = 'off';
  }
  if (field.has_default && !field.sensitive && field.default !== undefined) {
    input.placeholder = field.default;
  }
  return input;
}

/// Render the fields an app asks for, or nothing at all when it asks for none.
export function renderSetupFields(container, review) {
  const fields = review?.fields ?? [];
  if (!fields.length) return;
  const section = document.createElement('section');
  section.id = CONTAINER;
  const heading = document.createElement('h3');
  heading.className = 'section-label';
  heading.textContent = 'Setup';
  section.append(heading);

  if (review.generated_credential_count > 0) {
    const generated = document.createElement('p');
    generated.className = 'field-hint';
    const count = review.generated_credential_count;
    generated.textContent = count === 1
      ? 'One password is generated for this app and kept with its data.'
      : `${count} passwords are generated for this app and kept with its data.`;
    section.append(generated);
  }

  for (const field of fields) {
    const control = controlFor(field);
    control.dataset.setupKey = field.key;
    if (field.required) {
      control.required = true;
      control.setAttribute('aria-required', 'true');
    }
    const row = document.createElement('div');
    row.className = 'setup-row';
    row.append(labelled(field, control));
    const hint = hintFor(field);
    if (hint) {
      const note = document.createElement('small');
      note.className = 'field-hint';
      note.id = `${control.id}-hint`;
      note.textContent = hint;
      control.setAttribute('aria-describedby', note.id);
      row.append(note);
    }
    section.append(row);
  }
  container.append(section);
}

/// The first required field left empty, or null when the form is complete.
///
/// Only emptiness is checked here. Everything else — lengths, patterns, ranges
/// — is decided by the backend, which is the only place that can be trusted to
/// agree with what the install will actually accept.
export function firstMissingAnswer() {
  for (const control of document.querySelectorAll('[data-setup-key]')) {
    if (control.required && control.type !== 'checkbox' && !control.value.trim()) return control;
  }
  return null;
}

/// The answers as typed, with blanks left out so the backend applies its own
/// default rather than being handed an empty string that means something else.
export function collectAnswers() {
  const answers = {};
  for (const control of document.querySelectorAll('[data-setup-key]')) {
    const key = control.dataset.setupKey;
    if (control.type === 'checkbox') {
      answers[key] = String(control.checked);
      continue;
    }
    const value = control.value.trim();
    if (value) answers[key] = value;
  }
  return answers;
}

/// Mark the control an error names, so a message about `SITE_NAME` points at
/// the field the person filled in rather than at the form as a whole.
export function markInvalidAnswer(messageText) {
  for (const control of document.querySelectorAll('[data-setup-key]')) {
    control.removeAttribute('aria-invalid');
  }
  const named = [...document.querySelectorAll('[data-setup-key]')]
    .find(control => messageText.includes(control.dataset.setupKey));
  if (named) {
    named.setAttribute('aria-invalid', 'true');
    named.focus();
  }
  return Boolean(named);
}

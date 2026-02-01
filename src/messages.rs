use const_format::formatcp;

use crate::NAME;

pub const UNCHECKED: &str = formatcp!(
    "\
    ### ⏩ {NAME}\n\
    \n\
    - [ ] Check this box to fast-forward merge this PR\n\
    "
);

pub const CHECKED: &str = formatcp!(
    "\
    ### ⏩ {NAME}\n\
    \n\
    - [x] Check this box to fast-forward merge this PR\n\
    "
);

pub const MERGED: &str = formatcp!(
    "\
    ### ⏩ Successfully merged by {NAME}\n\
    "
);

pub const NOT_MERGEABLE: &str = formatcp!(
    "\
    ### ⏩ {NAME}\n\
    \n\
    ⚠️ This PR cannot be fast-forward merged. Please rebase your branch.\n\
    "
);

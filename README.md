NAME is a library with `Size` and `Duration` types specifically designed to be used in configuration files and as command line arguments.
These types serialize sizes and durations in _exact_ but human-readable form.

The library also provides `format_size` and `format_duration` functions to print _approximate_ sizes and durations in a short human-readable form.

The library does not use floating point operatins and suppports `no_std`.

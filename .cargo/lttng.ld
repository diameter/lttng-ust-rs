SECTIONS
{
    .lttng_ust_tracepoints_ptrs :
    {
        KEEP(*(.lttng_ust_tracepoints_ptrs))
        KEEP(*(.lttng_ust_tracepoints_ptrs.*))
    }
}
INSERT AFTER .data;

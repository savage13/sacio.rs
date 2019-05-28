
/*!
Extra documentation for sacio

   Field   | get | set
  -------  |-----|-----
  delta    | delta | -
  depmin   | min_amp   | extrema_amp / calc_min_amp
  depmax   | max_amp   | extrama_amp / calc_max_amp
  scale    | direct   | direct
  odelta   | direct   | direct
  b        | b   | set_b
  e        | e   | automatically computed
  o        | o   | set_o
  a        | direct   | direct
  fmt      | direct   | direct
  t0       | direct   | direct
  t1       | direct   | direct
  t2       | direct   | direct
  t3       | direct   | direct
  t4       | direct   | direct
  t5       | direct   | direct
  t6       | direct   | direct
  t7       | direct   | direct
  t8       | direct   | direct
  t9       | direct   | direct
  f        | direct   | direct
  resp0    | direct   | direct
  resp1    | direct   | direct
  resp2    | direct   | direct
  resp3    | direct   | direct
  resp4    | direct   | direct
  resp5    | direct   | direct
  resp6    | direct   | direct
  resp7    | direct   | direct
  resp8    | direct   | direct
  resp9    | direct   | direct
  stla     | station_lat       | set_station_location
  stlo     | station_lon       | set_station_location
  stel     | station_elevation | set_station_location
  stdp     | direct            | unavailable
  evla     | event_lat         | set_event_location
  evlo     | event_lon         | set_event_location
  evel     | direct            | unavailable
  evdp     | event_depth       | set_event_location
  mag      | direct   | direct
  user0    | direct   | direct
  user1    | direct   | direct
  user2    | direct   | direct
  user3    | direct   | direct
  user4    | direct   | direct
  user5    | direct   | direct
  user6    | direct   | direct
  user7    | direct   | direct
  user8    | direct   | direct
  user9    | direct   | direct
  dist     | dist_km   | compute_dist_az (evlo,evla,stlo,stla)
  az       | az   | compute_dist_az (evlo,evla,stlo,stla)
  baz      | baz   | compute_dist_az (evlo,evla,stlo,stla)
  gcarc    | dist_deg   | compute_dist_az (evlo,evla,stlo,stla)
  sb       | -   | -
  sdelta   | -   | -
  depmen   | mean_amp   | extrema_amp / calc_mean_amp
  cmpaz    | cmpaz   | set_cmpaz
  cmpinc   | cmpinc  | set_cmpinc
  xminimum | -   | -
  xmaximum | -   | -
  yminimum | -   | -
  ymaximum | -   | -
  nzyear   | time   | set_time
  nzjday   | time   | set_time
  nzhour   | time   | set_time
  nzmin    | time   | set_time
  nzsec    | time   | set_time
  nsmsec   | time   | set_time
  nvhdr    | header_version   | -
  norid    | int   | set_int
  nevid    | int   | set_int
  npts     | npts   | -
  nsnpts   | -   | -
  nwfid    | int   | set_int
  nxsize   | -   | -
  nysize   | -   | -
  iftype   | file_type   | set_file_type
  idep     | amp_type  | set_amp_type
  iztype   | zero_time_type   | set_zero_time_type
  iinst    | instrument_type  | set_intrument_type
  istreg   | station_region   | update_regions
  ievreg   | event_region   | update_regions
  ievtyp   | event_type   | set_event_type
  iqual    | data_quality   | set_data_quality
  isynth   | synthetic   | set_synthetic
  imagtyp  | magnitude_type   | set_magnitude_type
  imagsrc  | magnitude_source   | set_magnitude_source
  leven    | evenly_spaced   | -
  lpspol   | station_polarity   | set_station_polarity
  lovrok   | mutability   | set_mutability
  lcalda   | calc_dist_az   | set_calc_dist_az
  kstnm    | string | set_string
  kevnm    |  string | set_string
  khole    |  string | set_string
  ko       |  string | set_string
  ka       |  string | set_string
  kt0      | string | set_string
  kt1      | string | set_string
  kt2      | string | set_string
  kt3      | string | set_string
  kt4      | string | set_string
  kt5      | string | set_string
  kt6      | string | set_string
  kt7      | string | set_string
  kt8      | string | set_string
  kt9      | string | set_string
  kuser0   | string | set_string
  kuser1   | string | set_string
  kcmpnm   | string | set_string
  knetwk   | string | set_string
  kdatrd   | string | set_string
  kinst    |  string | set_string


*/

# Notes
#
Denny-Mumford wetted area:
w = Lwl*(1.7*T+Bwl*Cb)
Froude wetted area:
w = 3.4*(Lwl*Bwl*T*Cb)^(2/3)+.485*Lwl*(Lwl*Bwl*T*Cb)^(1/3)

Froude number:
Fn = V / pow(g * Lwl, 0.5)

Reynolds number:
Re = (V * Lwl) / ( Kinetic viscosity of water or 1.1e-6 kg/ms )

- Cargo ship D_lite needs to *not* include cargo

POUND2TON = 2240
function PERCENT_CALC(portion, total) # {{{1
    if (total > 0)
        return portion / total
    else
        return 0

function YEAR_ADJ(year) # {{{1
    if (year <= 1890)
        return 1 - (1890 - year) / 66.666664
    else if (year <= 1950)
        return 1
    else
        return 0

# Hull Page {{{1

# Errors {{{2
#

if (Cb > 0.3 && Cb <= 1.0)
    "Cb Coefficient must be 0.3 to 1.00"

# Freeboard Page {{{1

# Errors {{{2

if (lengthBow > -90.0 && lengthBow < 90f)
    "Bow angle cannot be 90 degrees or more"
if (free.fc_len > 1)
    "Forecastle too long"
if (free.fd_len > 1)
    "Forward deck too long"
if (free.qd_len > 1)
    "Quarter deck too long"
if (free.ad_len > 1)
    "Forecastle + forward deck + Quarter deck too long"

# Variables {{{2

free.FC_LEN_DFLT = 0.2
free.FD_LEN_DFLT = 0.3
free.AD_LEN_DFLT = 0.15

# Functions {{{2

function estFlushDeck # {{{3
    # THIS IS NOT A REAL FUNCTION IN SS3
    freeEstBow = (1.1 - (1 - YEAR_ADJ(year_laid)) * 0.5) * sqrt(Lwl)
    freeEstOther = (0.7 * sqrt(Lwl))

    half = (freeEstBow + freeEstOther) / 2

    fc_fwd = round(freeEstBow, 2)
    fc_aft = round(half, 2)
    fd_fwd = round(half, 2)
    fd_aft = round(freeEstOther, 2)
    ad_fwd = round(freeEstOther, 2)
    ad_aft = round(freeEstOther, 2)
    qd_fwd = round(freeEstOther, 2)
    qd_aft = round(freeEstOther, 2)
    
function estBreakDeck # {{{3
    # THIS IS NOT A REAL FUNCTION IN SS3
    freeEstBow = (1.1 - (1 - YEAR_ADJ(year_laid)) * 0.5) * sqrt(Lwl)
    freeEstOther = 0.9 * sqrt(Lwl)

    fc_fwd = round(freeEstBow, 2)
    fc_aft = round(freeEstOther, 2)
    fd_fwd = round(freeEstOther, 2)
    fd_aft = round(freeEstOther, 2)
    ad_fwd = round(freeEstOther / 2, 2)
    ad_aft = round(freeEstOther / 2, 2)
    qd_fwd = round(freeEstOther / 2, 2)
    qd_aft = round(freeEstOther / 2, 2)

function freeboard_description # {{{3
    # THIS IS NOT A REAL FUNCTION IN SS3

    if (fc_aft == fd_fwd && fd_aft == ad_fwd && ad_aft == qd_fwd)
        "Hull has a flush deck"
    else
        if (fc_aft != fd_fwd)
            if (fc_aft > fd_fwd)
                "raised forecastle"
            else if (fc_aft < fd_fwd)
                "low forecastle"
        if (fd_aft != ad_fwd)
            if (fd_aft > ad_fwd)
                "rise forward of midbreak"
            else if (fd_aft < ad_fwd)
                "rise aft of midbreak"
        if (ad_aft > qd_fwd)
            "low quarterdeck "
        if (ad_aft < qd_fwd)
            "raised quarterdeck "

# Guns Page {{{1

# Errors {{{2

if (gun.wgt_shell > gun.wgt_shell_est * 1.5)
    "Warning: Shell weight too heavy"
if (gun.wgt_shell < gun.wgt_shell_est * 0.5)
    "Warning: Shell weight too light"
if (D > wgt_broad / 4)
    "Design Failure: Reduce guns or increase Displacement"
if (gun.diameter > lookup(gun.type, gun_types[type], gun_types[max_d])
    "Warning: Gun diameter too large"
if (gun.year < lookup(gun.type, gun_types[type], gun_types[min_year])
    "Warning: Gun is too early"
if (gun.mount_type < lookup(gun.mount_type, mount_types[type], mount_types[min_year])
    "Warning: Mount type is too early"
if (gun.mount_type is not compatible with gun.type)
    "gun and mount combo is not allowed"
if ((gun.mount_type == BROADSIDE || gun.mount_type == CASEMATE ) && gun.num != gun.num_mounts)
    "Warning: No. of mounts must equal No. of gun - " + gunName
if (mount_type == BROADSIDE && distribution_type ~= CENTERLINE)
    "Warning: Centrally mounted broadsides not allowed"
if (mount_type == COLE, BARBETTE, TURRET && guns are below or sub)
    "Warning: below deck guns are not allowed"
if (mount type does not allow sub/below guns)
    "Warning: mount cannot be below freeboard deck - " + gunName
if (armor values missing / not allowed)
    "Warning: Armour too thick for mount"
    "Warning: Armour too thick for type"
    "Warning: mount cannot have below deck armour"
    "Warning: mount cannot have above deck armour"
    "Warning: Low broadside gun only have face armour"
    "Warning: Turret mount must have face armour"
    "Warning: mount must have barbette armour"
if (too many guns mounted)
    "Warning: Too many mounts in specific locations"

# Variables {{{2

CORDITE_FACTOR = 0.2444444
DEFAULT_CALIBER = 45

gun.year = main.dateCalc((dateBox).get_Text)
gun.g1_wgt_adj = weightAdjCalc(gun.g1_layout) # not a function anymore
gun.g2_wgt_adj = weightAdjCalc(gun.g2_layout) # not a function anymore
gun.wgt_adj = (gun.g1_wgt_adj * gun.g1_num_mounts + gun.g2_wgt_adj * gun.g2_num_mounts) /
    (gun.g1_num_mounts + gun.g2_num_mounts)

gun.g1_diameter_calc = diameter_calc(gun.g1_layout, gun.diameter, gun.g1_wgt_adj)
gun.g2_diameter_calc = diameter_calc(gun.g2_layout, gun.diameter, gun.g2_wgt_adj)

# OR the value for *all* batteries
cap_calc_broadside = if(mount_type == BROADSIDE, or(g1_below, g2_below, g2_lower, 0), false)

gun.date_factor = pow(YEAR_ADJ(gun.year), 0.5)
gun.wgt_shell_est =
    (
        (pow(gun.diameter, 3) / 1.9830943211886) *
        (1+if(gun.caliber<45, -1, 1)*pow(abs(45-gun.caliber), 0.5)/45)
    )*gun.date_factor
gun.wgt_magazine = (gun.shells_per * gun.num) * gun.wgt_shell / POUND2TON * (1+CORDITE_FACTOR)
gun.wgt_broad = gun.num * gun.wgt_shell
gun.wgt_gun = gun.wgt_shell * (gun.caliber / 812.389434917877 * (1 + pow(1 / gun.diameter, 2.3297949327695))) * gun.num
gun.wgt_mount = mount_wgt_calc(wgt_gun, gun.diameter, gun.caliber, gun.num, gun.type, gun.mount, gun.mount_adj, gun.wgt_adj)
gun.g1_guns_per = lookup(gun.g1_layout, layout_types[type], layout_types[guns])
gun.g1_guns.per = lookup(gun.g2_layout, layout_types[type], layout_types[guns])
gun.g1_num_mounts = gun.g1_super + gun.g1_above + gun.g1_deck + gun.g1_below + gun.g1_sub
gun.g2_num_mounts = gun.g2_super + gun.g2_above + gun.g2_deck + gun.g2_below + gun.g2_sub

super =
    (
        (
            (
                (2*gun.g1_super + gun.g1_above - 2*gun.g1_sub - gun.g1_below) * gun.g1_guns_per +
                (2*gun.g2_super + gun.g2_above - 2*gun.g2_sub - gun.g2_below) * gun.g2_guns_per
            ) / gun.num
        ) * max(7.5, gun.diameter*0.6) + gun.free
    ) / gun.free

gun.free = (gun.g1_free * gun.g1_num_mounts + gun.g2_free * gun.g2_num_mounts) / (gun.g1_num_mounts + gun.g2_num_mounts)

gun.g1_free = free_gun_calc((layoutG1).get_SelectedIndex, gun.g1_num_mounts)
gun.g2_free = free_gun_calc((layoutG2).get_SelectedIndex, gun.g2_num_mounts)

gun.armor_face_wgt = armor_face_wgt(gun.mount_type, gun.type, gun.armor_face)
gun.armor_back_wgt = armor_face_wgt(gun.mount_type,           gun.armor_back)
gun.armor_barb_wgt = armor_face_wgt(gun.mount_type,           gun.armor_barb)
gun.wgt_armor = gun.armor_face_wgt + gun.armor_back_wgt + gun.armor_bar_wgt

gun.concentration = CONCENTRATION_CALC(gun.shell_wgt, gun.num, gun.mount_adj, gun.num_mounts, gun.wgt_broad)

gun.g1_position = lookup(gun.g1_distribution, gun_distribution_types[type], gun_distribution_types[g1_gun_position])
gun.g2_position = lookup(gun.g2_distribution, gun_distribution_types[type], gun_distribution_types[g2_gun_position])

# Get these from layoutCalc
superAftG1 = superAft from layoutCalc()
positionG1 = gunPosition from layoutCalc()
superAftG2 = superAft from layoutCalc()
positionG2 = gunPosition from layoutCalc()

# Functions {{{2

function CONCENTRATION_CALC(shell_wgt, number,  mount_adj, mounts, broadside) # {{{3
    return (shell_wgt * number / broadside) * if(mount_adj>0.6, pow(4/mounts, 0.25) - 1, -0.1)

function mount_wgt_calc(gun_wgt, diameter, caliber, num, gun_type, mount_type, mount_adj, wgt_adj) # {{{3
    mount_wgt = lookup(mount_type, mount_types[type], mount_type[wgt])
       
    if (mount_adj < 0.6)
        mount_wgt *= lookup(gun_type, gun_types[type], gun_types[wgt_sm])
    else
        mount_wgt *= lookup(gun_type, gun_types[type], gun_types[wgt_lg])

    mount_wgt += 1f / pow(diameter, 0.313068808543972)
    mount_wgt *= gun_wgt

    if (diameter > 10)
        mount_wgt *= 1 - 2.1623769 * diameter / 100

    if (diameter <= 1)
        mount_wgt = gun_wgt

    mount_wgt *= wgt_adj

function diameter_calc(layout, diameter, wgt_adj) # {{{3
    num  = lookup(layout, gun_layout_types[type], gun_layout_types[num])
    num2 = lookup(layout, gun_layout_types[type], gun_layout_types[num2])

    num * diameter * (1 + pow(1/diameter, num2)

    if (diameter < 12)
        diameterGun += 12 / diameter
    if (diameter > 1 && weightAdj < 1)
        diameterGun *= 0.9

    return diameterGun

function free_gun_calc(distribution, num_mounts) # {{{3
    if (
                                                               distribution == 3, 4, 5, 6, 12, 13, 14 ||
        num_mounts == 1 &&                                     distribution == 1, 10   ||
        num_mounts == 1 && free.fc_len + free.fd_len >= 0.5 && distribution == 0,9
        )

        mounts_fwd = num_mounts
    else if (
                                                distribution == 1, 10 ||
            free.fc_len + free.fd_len >= 0.5 && distribution == 0, 9
            )

        mounts_fwd = round(num_mounts * 0.5, 0)
   else if (
                                                                  distribution == 6, 7, 8, 15, 16, 17 ||
            num_mounts == 1 &&                                    distribution == 2, 11 ||
            num_mounts == 1 && free.fc_len + free.fd_len < 0.5 && distribution == 0, 9
            )

        mounts_fwd = 0f
    else
        mounts_fwd = num_mounts - round(num_mounts * 0.5, 0)

    if (num_mounts > 0)
        switch (distribution)
        case 0:
        case 9:
            gun.free = (mounts_fwd * free.fd + (num_mounts - mounts_fwd) * free.ad) / num_mounts
        case 1:
        case 2:
        case 10:
        case 11:
            if (mounts_fwd > 0)
                gun.free = mounts_fwd * ((free.fd_fwd - free.fd) / mounts_fwd * 0.5 + (free.fd_fwd + free.fd) * 0.5)
            if (num_mounts - mounts_fwd > 0)
                gun.free += (num_mounts - mounts_fwd) * (
                    (free.ad_aft - free.ad) * 1 / (num_mounts - mounts_fwd) * 0.5 + (free.ad_aft + free.ad) * 0.5
                    )
            gun.free /= num_mounts
        case 3:
        case 12:
            if (mounts_fwd > 0)
                gun.free = (free.fd_fwd - free.fd) / mounts_fwd * 0.5 + (free.fd_fwd + free.fd) * 0.5
        case 4:
        case 13:
            gun.free = free.fd
        case 5:
        case 14:
            if (mounts_fwd > 0)
                gun.free = (free.fd_aft - free.fd) / mounts_fwd * 0.5 + (free.fd_aft + free.fd) * 0.5
        case 6:
        case 15:
            if (num_mounts - mounts_fwd > 0)
                gun.free = (free.ad_fwd - free.ad) / (num_mounts - mounts_fwd) * 0.5 + (free.ad_fwd + free.ad) * 0.5
        case 7:
        case 16:
                gun.free = free.ad
        case 8:
        case 17:
            if (num_mounts - mounts_fwd > 0)
                gun.free = (free.ad_aft - free.ad) / (num_mounts - mounts_fwd) * 0.5 + (free.ad_aft + free.ad) * 0.5

function armor_face_wgt(mount, gun, thick) # {{{3
    wgt = lookup(mount, mount_types[type], mount_types[armor_face_wgt]) +
          (gun.armor_face == 0 ? lookup(mount, mount_types[type], mount_types[armor_face_wgt_if_no_back] : 0)

    wgt *= (
        gun.g1_diameter_calc * gun.g1_num_mounts +
        gun.g2_diameter_calc * gun.g2_num_mounts
        ) * gun.house_hgt * thick * INCH

    wgt *= lookup(gun, gun_types[type], gun_types[armor_face_wgt]) * 
           lookup(gun, gun_types[type], gun_types[armor_face_wgt_if_no_back])

function armor_back_wgt(mount, thick) # {{{3
    BACK = pi * pow(gun.g1_diameter_calc / 2, 2.0) * gun.g1_num_mounts +
        pi * pow(gun.g2_diameter_calc / 2, 2.0) * gun.g2_num_mounts

    wgt = lookup(mount, mount_types[type], mount_types[armor_back_wgt])
    wgt *= (gun.g1_diameter_calc * gun.g1_num_mounts + gun.g2_diameter_calc * gun.g2_num_mounts) * gun.house_hgt
    wgt += BACK * (mount == 1 ? 1 : 0.75)
    wgt *= thick * INCH

    wgt = 2.5 * (gun.g1_diameter_calc * gun.g1_num_mounts + gun.g2_diameter_calc * gun.g2_num_mounts) * gun.houst_hgt +
          (
            (
                pi() * pow(gun.g1_diameter_calc / 2, 2) * gun.g1_num_mounts +
                pi() * pow(gun.g2_diameter_calc / 2, 2) * gun.g2_num_mounts
            ) * 0.75
          ) * thick * INCH

function armor_barb_wgt(mount, thick) # {{{3
    guns_per = (gun.g1_guns_per * gun.g1_num_mounts + gun.g1_guns.per * gun.g2_num_mounts) / (gun.g1_num_mounts + gun.g2_num_mounts)

    # XXX: The original code set gun.g1_guns_per if the condition failed which seems wrong. See armWeightCalc()
    guns_per = min((gun.mount_adj > 0.5 ? 4 : 5), guns_per)

    wgt = lookup(mount, mount_types[type], mount_types[armor_barb_wgt]

    wgt = (1 - (guns_per - 2) / 6) *
          (
            thick * gun.num * pow(gun.diameter, 1.2) * wgt * gun.free / 16 *
            gun.super * wgt * 2 * sqrt(gun.date_factor)
          )

    # XXX: Can this condition ever actually fail?
    wgt = (gun.free > 0 ? wgt : 0)

function broadsideCalc # {{{3
    if (gun_armor_wgt + gunsWeight > 0f)
        superFactor =
            (
                gun.wgt_gun+gun.wgt_mount+gun.wgt_armor)
                gun1.totalWeight * gun1.super * gun1.mountAdj +
                gun2.totalWeight * gun2.super * gun2.mountAdj +
                gun3.totalWeight * gun3.super * gun3.mountAdj +
                gun4.totalWeight * gun4.super * gun4.mountAdj +
                gun5.totalWeight * gun5.super * gun5.mountAdj
            ) / (wgt_gun_armor + wgt_guns);
    else
        superFactor = 1

function layoutCalc(int gunLayout, int gunMounts, int group) # {{{3
    superAft = false
    switch (gunLayout)
    case 0:
        if (gunMounts == 1)
            if ((free.free.fc_len + free.free.fd_len) >= 0.5)
                layoutText = "on centreline amidships (forward deck)"
            else
                layoutText = "on centreline amidships (aft deck)"
        else
            layoutText = "on centreline, evenly spread"
    case 1:
        if (gunMounts == 1)
            layoutText = "on centreline forward"
        else if (gunMounts % 2 == 0)
            layoutText = "on centreline ends, evenly spread"
        else
            layoutText = "on centreline ends, majority forward"
    case 2:
        if (gunMounts == 1)
            layoutText = "on centreline aft"
        else if (gunMounts % 2 == 0)
            layoutText = "on centreline ends, evenly spread"
        else
            layoutText = "on centreline ends, majority aft"
        superAft = true
    case 3:
        layoutText = "on centreline, forward deck forward"
        gunPosition = 0.25f * free.free.fd_len
    case 4:
        if (gunMounts == 1)
            layoutText = "on centreline, forward deck centre"
        else
            layoutText = "on centreline, forward evenly spread"
        gunPosition = 0.5f * free.free.fd_len
    case 5:
        layoutText = "on centreline, forward deck aft"
        gunPosition = 0.75f * free.free.fd_len
    case 6:
        layoutText = "on centreline, aft deck forward"
        superAft = true
        gunPosition = 0.25f * free.free.ad_len
    case 7:
        if (gunMounts == 1)
            layoutText = "on centreline, aft deck centre"
        else
            layoutText = "on centreline, aft evenly spread"
        superAft = true
        gunPosition = 0.5f * free.free.ad_len
    case 8:
        layoutText = "on centreline, aft deck aft"
        superAft = true
        gunPosition = 0.75f * free.free.ad_len
    case 9:
        if (gunMounts < 3)
            layoutText = "on sides amidships"
        else
            layoutText = "on sides, evenly spread"
    case 10:
        if (gunMounts < 3)
            layoutText = "on sides forward"
        else if (gunMounts % 4 == 0)
            layoutText = "on side ends, evenly spread"
        else
            layoutText = "on side ends, majority forward"
    case 11:
        if (gunMounts < 3)
            layoutText = "on sides aft"
        else if (gunMounts % 4 == 0)
            layoutText = "on side ends, evenly spread"
        else
            layoutText = "on side ends, majority aft"
        superAft = true
    case 12:
        layoutText = "on sides, forward deck forward"
        gunPosition = 0.25f * free.free.fd_len
    case 13:
        if (gunMounts < 3)
            layoutText = "on sides, forward deck centre"
        else
            layoutText = "on sides, forward evenly spread"
        gunPosition = 0.5f * free.free.fd_len
    case 14:
        layoutText = "on sides, forward deck aft"
        gunPosition = 0.75f * free.free.fd_len
    case 15:
        layoutText = "on sides, aft deck forward"
        superAft = true
        gunPosition = 0.25f * free.free.ad_len
    case 16:
        if (gunMounts < 3)
            layoutText = "on sides, aft deck centre"
        else
            layoutText = "on sides, aft evenly spread"
        superAft = true
        gunPosition = 0.5f * free.free.ad_len
    case 17:
        layoutText = "on sides, aft deck aft"
        superAft = true
        gunPosition = 0.75f * free.free.ad_len
    default:
        layoutText = "layout not set"
    if (gunLayout <= 2 || (gunLayout >= 9 && gunLayout <= 11) || gunLayout > 17)
        if (group == 1)
            gunPosition = 1f
        else
            gunPosition = 0f

# Armor Page {{{1

# Errors {{{2

if (armor.belt_main_len + armor.belt_end_len > Lwl
    "Main + End armor belts too long"
if (armor.belt_up_len > Lwl)
    "Upper armor belt too long"
if (armor.belt_main_hgt > armor.belt_hgt_max ||
    "Main belt too tall"
if (armor.belt_end_hgt > armor.belt_hgt_max ||
    "End belt too tall"
if (armor.belt_up_hgt > freeboard_dist + 0.01)
    "Upper belt too tall"
if (armor.bh_hgt > T + freeboard_dist)
    "Torpedo bulkhead too tall"
if (armor.bulge_hgt > T + freeboard_dist)
    "Torpedo bulge too tall"
if (wgt_armor < D)
    "Design Failure: Reduce armour or increase Displacement"
# I changed this from ">=" to ">" for simplicity
if (armor.bh_beam > lookup(armor.bh_type, bulkhead_types[type], bulkhead_types[max_len])
        "Warning: Beam between bulkheads too wide"

# Functions {{{2

function armorEst # {{{3
    # THIS IS NOT A REAL SS3 FUNCTION
    EstMainLen= (1 - free.fc_len - free.qd_len) * Lwl
    EstMainHgt = min(1.2 * sqrt(B), Ts + freeboard_dist)
    EstEndLen = Lwl - beltMainLengthEst - 0.02
    EstUpHgt = min(8, freeboard_dist)
    EstBulkHgt = TSide

    main_len = EstMainLen
    main_hgt = EstMainHgt
    end_len  = EstEndLen
    end_hgt  = EstMainHgt
    up_len   = EstMainLen
    up_hgt   = EstUpHgt
    bulk_len = EstMainLen
    bulk_hgt = EstBulkHgt

# Weapons Page {{{1

# Errors {{{2

# Variables {{{2

weight = torp1.weaponWeight + torp1.mountWeight +
         torp2.weaponWeight + torp2.mountWeight +
         mine1.weaponWeight + mine1.mountWeight +
         dc1.weaponWeight   + dc1.mountWeight   +
         dc2.weaponWeight   + dc2.mountWeight

# Engine Page {{{1

# Errors {{{2

if (coal_pct <= 100f && coal_pct >= 0f)
    "Percentage coal must be between 0% and 100%"
if (hp_max / num_shafts > MAX_HP_PER_SHAFT && !(engine == "turbine" && (engine == "recpirocating"))
    "Warning: Too much power for reciprocating engines"
if (hp_max / num_shafts > MAX_HP_PER_SHAFT_RECIP)
    "Warning: Too much power for number of propellor shafts"
if (year_engine < 1898 && fuel == "oil")
    "Warning: Too early for oil fired boilers"
if (year_engine < 1904 && fuel == "diesel")
    "Warning: Too early for diesel motors"
if (year_engine < 1898 && fuel == "gasoline")
    "Warning: Too early for gasoline motors"
if (year_engine < 1898 && fuel == "battery")
    "Warning: Too early for batteries"
if (year_engine < 1898 && engine == "turbine")
    "Warning: Too early for steam turbines"
if (year_engine < 1885 && engine == "complex")
    "Warning: Too early for complex reciprocating steam engines"
if (year_engine < 1911 && drive == "geared")
    "Warning: Too early for geared drives"
if (year_engine < 1898 && drive == "electric")
    "Warning: Too early for electric drives"

# Variables {{{2

MAX_HP_PER_SHAFT       = 75000
MAX_HP_PER_SHAFT_RECIP = 20000

engine.bunker = bunkerCalc()

wgt_guns       = sum(gun.wgt_gun)
wgt_gun_mounts = sum(gun.wgt_mounts)
wgt_gun_armor  = sum(gun.wgt_armor)
wgt_mag        = sum(gun.wgt_magazine)
wgt_borne      = sumproduct(gun.wgt_gun, gun.mount_adj) * 2
wgt_broad      = sum(gun.wgt_broad)

if (wgt_gun_armor + wgt_guns + wgt_gun_mounts > 0f)
    superFactor = (gun1.totalWeight * gun1.super * gun1.mountAdj + gun2.totalWeight * gun2.super * gun2.mountAdj + gun3.totalWeight * gun3.super * gun3.mountAdj + gun4.totalWeight * gun4.super * gun4.mountAdj + gun5.totalWeight * gun5.super * gun5.mountAdj) / (wgt_gun_armor + wgt_guns + wgt_gun_mounts);
				arm.weightCalc();
else
    superFactor = 1f;


wgt_hull      = D - wgt_guns - wgt_gun_mounts - wgt_weaps - wgt_weaps_mounts - wgt_armor - wgt_engine - wgt_load - wgt_misc
wgt_hull_plus = wgt_hull + wgt_guns + wgt_gun_mounts - wgt_borne

# simple engine => simple
# complex engine => complex
# turbine || diesel || gas || battery => other
wgt_engine = engineCalc()

hullWeight = hull.displacement - guns.gunsWeight - weapons.weight - arm.weight - engineWeight - loadWeight - weapons.miscWeight;

# Functions {{{2

function engineLayout # {{{3
    # NOTE: NOT COMPLETELY IMPLEMENTED
    # THIS IS JUST DISPLAYING TEXT
    if (!fuelArray[0] && !fuelArray[1] && !fuelArray[2] && !fuelArray[3] && !fuelArray[4])
        fuelString = "No fuel, "
    else if (fuelArray[0] && !fuelArray[1] && !fuelArray[3])
        fuelString = "Coal fired boilers, "
    else if (!fuelArray[0] && fuelArray[1] && !fuelArray[3])
        fuelString = "Oil fired boilers, "
    else if (!fuelArray[0] && !fuelArray[1] && fuelArray[2] && !fuelArray[3])
        fuelString = "Diesel "
    else if (!fuelArray[0] && !fuelArray[1] && !fuelArray[2] && fuelArray[3])
        fuelString = "Petrol "
    else if (!fuelArray[0] && !fuelArray[1] && !fuelArray[2] && !fuelArray[3] && fuelArray[4])
        fuelString = "Battery powered "
    else if (fuelArray[0] && fuelArray[1] && !fuelArray[3])
        fuelString = "Coal and oil fired boilers, "
    else
        fuelString = "ERROR: Revise fuels, "
    if (fuelArray[0] || fuelArray[1])
        if (!engineArray[0] && !engineArray[1] && !engineArray[2])
            engineString1 = "ERROR: no steam engines, "
        else if (engineArray[0] && !engineArray[1] && !engineArray[2])
            engineString1 = "simple reciprocating steam engines, "
        else if (!engineArray[0] && engineArray[1] && !engineArray[2])
            engineString1 = "complex reciprocating steam engines, "
        else if (!engineArray[0] && !engineArray[1] && engineArray[2])
            engineString1 = "steam turbines, "
        else if (engineArray[0] && engineArray[1] && !engineArray[2])
            engineString1 = "reciprocating steam engines, "
        else if ((engineArray[0] || engineArray[1]) && engineArray[2])
            if (engineArray[0] && engineArray[1] && engineArray[2])
                engineString1 = "ERROR: Too many types of steam engines, "
            else
                engineString1 = "reciprocating cruising steam engines"
                engineString2 = " and steam turbines "
        else
            engineString1 = "ERROR: Revise fuels or engines, "
            engineString2 = ""

        if (fuelArray[2] || fuelArray[3] || fuelArray[4])
            if (fuelArray[4] && !fuelArray[2])
                engineString2 += " plus batteries, "
            else if (fuelArray[2] && !fuelArray[4])
                engineString2 += " plus diesel motors, "
            else
                engineString2 += " ERROR: Revise auxilliary fuel, "
    else if (fuelArray[4])
        if (!fuelArray[3] && !fuelArray[2])
            driveCheckedList.SetItemChecked(0, false)
            driveCheckedList.SetItemChecked(1, false)
            driveCheckedList.SetItemChecked(2, true)
            driveCheckedList.SetItemChecked(3, false)
            engineString1 = ""
        else if (driveArray[0] || driveArray[1] || driveArray[3])
            engineString1 = "Internal combustion engines plus batteries, "
        else
            engineString1 = "Internal combustion generators plus batteries, "
    else if ((fuelArray[3] || fuelArray[2]) && driveArray[2] && !driveArray[0] && !driveArray[1])
        engineString1 = "Internal combustion generators, "
    else
        engineString1 = "Internal combustion motors, "
    if (fuelArray[4] && fuelArray[3] && (fuelArray[0] || fuelArray[1]))
        engineString1 = "ERROR: Too many different fuels, "
    if (!driveArray[0] && !driveArray[1] && !driveArray[2] && !driveArray[3])
        driveString = "No drive to shaft"
    else if (driveArray[0] && !driveArray[1] && !driveArray[2] && !driveArray[3])
        driveString = "Direct drive"
    else if (!driveArray[0] && driveArray[1] && !driveArray[2] && !driveArray[3])
        driveString = "Geared drive"
    else if (!driveArray[0] && !driveArray[1] && driveArray[2] && !driveArray[3])
        driveString = "Electric motors"
    else if (!driveArray[0] && !driveArray[1] && !driveArray[2] && driveArray[3])
        driveString = "Hydraulic drive"
    else if (!driveArray[0] && driveArray[1] && driveArray[2] && !driveArray[3])
        driveString = "Electric cruising motors plus geared drives"
    else
        driveString = "ERROR: Revise drives"
    if (fuelArray[0])
        if (!fuelArray[1] && !fuelArray[2] && !fuelArray[3])
            (coal_pctBox).set_Text("100")
        if (coal_pct == 0f || (coal_pct == 100f && fuelArray[1]) || fuelArray[2] || fuelArray[3])
            "Enter percentage of bunker tonnage devoted to coal fired boilers")
        else
            "Enter percentage of bunker tonnage devoted to coal fired boilers"

# Performance Page {{{1

# Errors {{{2

if (steadiness >= 0 && steadiness <= 100)
    "Trim must be 1 to 100"

# Variables {{{2

cost_dollar = ((D - engine.wgt_load) * 0.00014 + wgt_engine * 0.00056 + (wgt_borne * 8) * 0.00042) *
    if(year_laid+2>1914, (1+(year_laid+1.5-1914)/5.5),1)
cost_lb = costDollars / 4

perf.room =
    (
        (wgt_mag + D*0.02 + wgt_borne * 6.4 + wgt_engine * 3 + wgt_misc.vital + wgt_misc.hull) /
        (D*0.94) / (1-hull_space)
    )
perf.hull_room = perf.room * (armor.bh_wgt>0.1 ? (B/armor.bh_beam) : 1)
perf.deck_room = WP / TON / 15 * (1 - deck_space) / crew_min * freeboard_dist

perf.wgt_struct =
    (
        engine.wgt_hull_plus + if(lookup(armor.bh_type, bulkhead_types[type], bulkhead_types[improve_structure], armor.bh_wgt, 0)
    ) * POUND2TON /
    (WS + 2 * Lwl * free.cap + WP)

perf.str_long =
    (
        wgt_hull + iferror(if(xlookup(armor.bh_type, bulkhead_types[type], bulkhead_types[improve_struct]), armor.bh_wgt, 0),0)
    ) /
    (
        pow(Lwl/(T+free.cap),2)*(D+armor.belt_end_wgt*3+(wgt_borne+wgt_gun_armor)*superFactorLong*2)
    )*850*(if(year_laid<1900,1-(1900-year_laid)/100,1))

perf.str_cross =
    (perf.wgt_struct /
    sqrt(Bb * (T + freeboard_dist)) /
    (
        (
            D +
            (
                (wgt_broad + wgt_borne + wgt_gun_armor + armor.ct_wgt) * ((1+concentration) * gun.super_factor) +
                max(engine.hp_max, 0) / 100
            )
        ) / D
    ) * 0.6) * (year_laid<1900 ? 1-(1900-year_laid)/100, 1)

perf.str_comp = perf.str_cross > perf.str_long ?
    perf.str_long * pow(perf.str_cross / perf.str_long, 0.25) :
    perf.str_cross * pow(perf.str_long / perf.str_cross, 0.1)

perf.damage_shell = perf.float / (pow((Battery 1 diameter > 0 ? Battery 1 diameter : 6), 3) / 2 * year_laid_adj
perf.damage_torp =
        (   
            (
                pow(perf.float / 10000, 1/3) +
                pow(Bb / 75, 2) +
                pow((armor.bh_thick / 2 * armor.bh_len / Lwl) / 0.65 * armor.bh_hgt / T, 1/3) *
                perf.float / 35000 * Bb / 50
            ) / perf.room * Lwl / (Lwl + Bb)
        ) * (perf.stability_adj<1 ? pow(perf.stability_adj, 4) : 1) *
        (1-hull_space) *
        (Main torpedo weight > 0 ? 1.313 / (main torpedo weight / main torpedo number) : 1))

perf.stability = stabilityCalc()
perf.stability_adj = perf.stability * ((50 - perf.trim) / 150 + 1)
perf.metacenter = pow(B, 1.5) * (perf.stability_adj - 0.5) / 0.5 / 200
perf.steadiness = min(perf.trim * perf.seaboat, 100)
perf.seakeeping = perf.seaboat * (perf.steadiness<50, perf.steadiness, 50) / 50
perf.seaboat = seaboatCalc()

perf.roll_period = 0.42 * Bb / sqrt(perf.metacenter)

perf.recoil = ((wgt_broad / D * freeboard_dist * gun.super_factor / Bb) * pow(pow(D, 1/3) / Bb * 3, 2) * 7) /
    (perf.stability_adj > 0 ? perf.stability_adj * ((50-perf.steadiness) / 150 + 1) : 1)

perf.float = flotationCalc()

# SS3         = springsheet         Usage
steadiness    = perf.trim           # Trim box
steadinessAdj = perf.steadiness     # Steadiness box
stabilityAdj  = perf.stability_adj  # Stability box
stability     = perf.stability      # Used internally
seaboat       = perf.seaboat        # Used internally
seakeeping    = perf.seakeep        # Seakeeping box

# Functions {{{2

function perf.messages # {{{3
    # NOT A SS3 FUNCTION
    if (perf.hull_room < 5 / 6)
        "Excellent machinery, storage, compartmentation space"
    else if (perf.hull_room < 1.1111112)
        "Adequate machinery, storage, compartmentation space"
    else if (perf.hull_room <= 2)
        "Cramped machinery, storage, compartmentation space"
    else
        "Extremely poor machinery, storage, compartmentation space"

    if (perf.room_deck > 1.2)
        "Excellent accommodation and workspace room"
    else if (perf.room_deck > 0.9)
        "Adequate accommodation and workspace room"
    else if (perf.room_deck >= 0.5)
        "Cramped accommodation and workspace room"
    else
        "Extremely poor accommodation and workspace room"

    if (perf.str_comp < 0.5)
        "DESIGN FAILURE: Hull structure insufficient to carry required loads!"
    else if (perf.str_comp < 0.995)
        if (Vm >= 24 && D < 4000)
            "Design well balanced for a light fast combatant"
        else
            "WARNING: Hull structure strained by open sea conditions! (Advisable for light fast combatants only)"
    else if (perf.str_comp < 1.005)
        "Design well balanced for a Battleship or Cruiser"
    else
        "Design undergunned or otherwise under performing relative to displacement"

    if (stabilityAdj <= 0.995)
        tenderWarn = true
        (stabilitylbl).set_ForeColor(Color.Red)
    else
        tenderWarn = false
        (stabilitylbl).set_ForeColor(SystemColors.ControlText)

    if (torp1.weaponWeight > 0)
        (damageTorplbl).set_Text("Max " + torp1.size.ToString("n2") + " \" / " + main.metricCalc(torp1.size, Con.inch2mm).ToString("n0") + "mm torpedo hits:")
    else
        (damageTorplbl).set_Text("Max 20\" / 508mm torpedo hits:")

    if (perf.steadiness >= 69.5)
        steady = true
        unsteady = false
        (steadinesslbl).set_ForeColor(Color.Green)
    else if (perf.steadiness < 30)
        steady = false
        unsteady = true
        (steadinesslbl).set_ForeColor(Color.Red)
    else
        steady = false
        unsteady = false
        (steadinesslbl).set_ForeColor(SystemColors.ControlText)




    badSea = (poorSea = (fineSea = (goodSea = false)))
    if (seakeeping < 0.7)
        badSea = true
        (seakeepinglbl).set_ForeColor(Color.Red)
    else if (seakeeping < 0.995)
        poorSea = true
        (seakeepinglbl).set_ForeColor(Color.DarkRed)
    else if (seakeeping >= 1.5)
        fineSea = true
        (seakeepinglbl).set_ForeColor(Color.Green)
    else if (seakeeping >= 1.2)
        goodSea = true
        (seakeepinglbl).set_ForeColor(Color.DarkGreen)
    else
        (seakeepinglbl).set_ForeColor(SystemColors.ControlText)


function stabilityCalc # {{{3
    stability =
        armor.ct_wgt * 5 +
        (wgt_borne + wgt_gun_armor) * (2 * gun.super_factor - 1) * 4 +
        wgt_misc.hull * 2 +
        wgt_misc.deck * 3 +
        wgt_misc.above * 4 +
        armor.belt_up_wgt * 2 +
        armor.belt_main_wgt +
        armor.belt_end_wgt + 
        armor.deck_wgt +
        (wgt_hull_plus + wgt_guns + wgt_gun_mounts - wgt_borne) * 1.5 * freeboard / T

    if (perf.room_deck < 1)
        stability += (wgt_engine + wgt_misc.vital + wgt_misc.void) * pow(1 - perf.room_deck, 2)

    if (stability > 0)
        stability = sqrt((D * (Bb / T) / stability) * 0.5) * pow(8.76755 / B, 0.25)

    return stability

function flotationCalc # {{{3
    if (gun.cap_calc_broadside)
        flotation = free.cap
    else
        flotation = freeboard_dist
    flotation  = (flotation * WP / TON + D) / 2
    flotation *= pow(perf.stability_adj, (perf.stability_adj > 1 ? 0.5, 4)
    flotation *= perf.str_comp < 1 ? perf.str_comp : 1
    flotation /= pow(perf.room, (perf.room > 1 ? 2 : 1)
    flotation *= year_laid_adj
    flotation = max(0, flotation)

function superFactorLongCalc # {{{3
    superFactorLong = perf.hull_room

    if ((gun.main_g1_disribution == 0, 9 || gun.main_g2_distribution == 0, 9) &&
         gun.main_num_mounts == 3, 4)
        superFactorLong *= gun.super_factor

    else if
        gun.main_g1_num_mounts > 0 && gun.main_g2_num_mounts == 0 && lookup(gun.main_g1_distribution, gun_distribution_types[type], gun_distribution_types[super_factor_long]) ||
        gun.main_g2_num_mounts > 0 && gun.main_g1_num_mounts == 1 && lookup(gun.main_g1_distribution, gun_distribution_types[type], gun_distribution_types[super_factor_long]) ||
        gun.main_g1_num_mounts > 0 && gun.main_g2_num_mounts > 0 && abs(main positionG1 - main positionG2) < 0.2

        superFactorLong *= 0.8 * gun.super_factor

    else
        superFactorLong *= 2 * gun.super_factor - 1

function seaboatCalc # {{{3
    perf.seaboat = sqrt(free.cap / (2.4 * pow(D, 0.2))) *
               (
                    pow(perf.stability * 5 * (Bb / Lwl), 0.2) *
                    sqrt(free.cap / Lwl * 20) *
                    (
                        D / (
                            D + armor.belt_end_wgt * 3 + wgt_hull /
                            3 + (wgt_borne + wgt_gun_armor) * superFactorLong
                        )
                    )
                ) * 8

    if ((T / Bb) < 0.3)
        perf.seaboat *= sqrt((T / Bb) / 0.3)

    if ((Rf_max / (Rf_max + Rw_max)) < 0.55 && Vm != 0)
        perf.seaboat *= pow(Rf_max / (Rf_max + Rw_max), 2)
    else
        perf.seaboat *= 0.3025

    perf.seaboat = min(perf.seaboat, 2)


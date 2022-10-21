use super::{nn, wall_sim::Builders};

#[allow(clippy::excessive_precision)]
pub const HP: nn::Hyperparams = nn::Hyperparams {
    init_fac: 1.0,
    bias_fac: 0.2, // FIXME: easy to forget updating, duplicated here and in .py script
};

// pub const PARAMS: Option<[f32; Builders::PARAM_COUNT]> = None;

// optimized by CMA-ES
#[allow(clippy::excessive_precision)]
pub const PARAMS: Option<[f32; Builders::PARAM_COUNT]> = Some([
    2.269507400673745678e+00,
    1.262969272607275073e+01,
    5.368680337583467299e+00,
    -2.216272970071416637e+01,
    -4.890271907373009874e+00,
    -1.954442271369721951e+01,
    -6.296446161173773426e+00,
    1.207247851761669821e+00,
    2.149869009359001026e+01,
    -2.217530075095350739e+01,
    -5.005073315228609765e+00,
    -8.679474260090783488e+00,
    4.393578978019734294e+00,
    1.294013218458741754e+01,
    1.443221113008527956e+00,
    -1.442880045373159348e+01,
    -2.865388584832226471e+00,
    -1.164774566743180273e+01,
    -1.646593963265420157e+01,
    7.983982072752029424e+00,
    3.851973007732404142e+00,
    -1.079644776247300619e+01,
    1.772644429025233492e+01,
    -1.226112434817092378e+01,
    4.082014018190254667e+00,
    1.407536979638040009e+01,
    2.114854759693351127e+00,
    2.326968845905095673e+00,
    -5.870913627405711654e-01,
    -1.210912590352280027e+01,
    -1.241701260927380979e+01,
    -1.935275444143090384e+00,
    7.921221197998139729e+00,
    3.099434046880834348e+00,
    -1.198185833825117008e+01,
    1.664872822273634867e+00,
    8.198313757856650064e-01,
    -1.861113124537936647e+00,
    -9.086237353613585199e+00,
    -6.071089247576795245e+00,
    -9.203494748611635856e+00,
    -1.303658777274868363e+01,
    -2.103469268431321026e+01,
    1.309020620256065115e+01,
    -1.198599582167266142e+01,
    7.290449641438152106e+00,
    -1.416575206147804966e+00,
    -2.196971683716405366e+01,
    1.389463720568170046e+01,
    6.710611175395451156e+00,
    2.691945745754243724e+00,
    1.310276963499908121e+01,
    -1.927866672889321009e+01,
    8.981647742311757998e-01,
    -1.450341530204336138e+01,
    2.664825853176740367e+01,
    -5.696265668864107568e+00,
    8.973985297493907609e+00,
    -1.083961063908986944e+01,
    -1.019267623294990344e+01,
    -8.726755412318000893e+00,
    -1.778055609792849046e+00,
    -5.657925637154464837e+00,
    4.399832657193964103e+00,
    -1.009682249092633377e+01,
    -8.019006641333039198e+00,
    -2.432815026767873334e+01,
    -5.881635064951572289e+00,
    -1.700162280056406772e+01,
    1.650226968936540306e+00,
    -2.144433814380817083e+01,
    -8.348290254785887265e-01,
    2.695091902441406617e+00,
    6.203436031714115728e-01,
    6.236870806086529662e-01,
    7.985177654141299985e-01,
    -9.222401821944151479e+00,
    -8.833291887519955088e-01,
    -1.131762643237526156e+01,
    -6.051755660187409624e+00,
    1.935651626958399429e+00,
    4.962635369234018157e+00,
    -1.279775333836403206e+01,
    -1.110960077589521156e+01,
    3.163973482091078981e+00,
    1.424328272384517291e+01,
    -2.215294009764199856e+01,
    4.603406947699984819e+00,
    -2.389393784863953041e+00,
    -1.402149989743665692e+01,
    -3.044671666244563202e+00,
    -1.302182774351225447e+01,
    5.161657181936883942e-01,
    -3.880781948400132464e+00,
    2.623800077523714247e+00,
    -9.480033655340054821e+00,
    -6.021160200469022783e+00,
    1.814033131239777941e+00,
    -8.400799003874436233e-01,
    -7.460880617007154214e+00,
    2.276990978691681711e-01,
    1.032561404390382798e+01,
    -3.767442561375220489e+00,
    -1.142466564099808402e+00,
    1.465765609842884665e+01,
    -2.404781848112209630e+00,
    -7.300334505423561993e+00,
    1.477833814910203891e+01,
    4.615398608984359718e+00,
    7.282536692973791415e-02,
    1.305033549741929555e+01,
    -1.965318023618018994e+00,
    -1.763853463284791534e+00,
    1.801188825422770634e+01,
    -1.678376630950493720e+01,
    -4.030025439760220429e+00,
    8.025449839896689141e+00,
    5.804180759109532239e+00,
    -9.471680108331613113e+00,
    -1.314210648408946724e+01,
    2.679564923096401152e+00,
    -5.074897538488020388e+00,
    8.826083701667876369e+00,
    1.149140684621345265e+01,
    9.701725345032304659e+00,
    8.598944950945490717e+00,
    -8.095982418816248583e+00,
    -1.282309371046264701e+01,
    1.744078606224846340e+01,
    -5.554251009835359021e+00,
    1.368615476162804079e+01,
    2.486733723030581711e+00,
    2.607566523958271265e+00,
    -5.709405387776920193e+00,
    1.003024797783700528e+01,
    -1.049349178275654459e+01,
    8.700179856156275982e-02,
    2.651123991710560546e+01,
    -1.264775619662636430e+01,
    -4.859150821167570911e+00,
    7.706911484221173225e+00,
    1.066853319637134945e+01,
    1.166236394817582456e-03,
    -1.848201548210139711e+00,
]);
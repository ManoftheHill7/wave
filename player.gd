extends CharacterBody2D


const ACCEL = 50.0
const SPEED = 300.0
const CLIMBSPEED = 100.0
const CLIMB_STAMINA = 2.0
const JUMP_VELOCITY = -300.0
const TERMINAL_VELOCITY = 600
const JUMP_GRAVITY_REDUCTION = 0.65
const JUMP_RELEASE_REDUCTION = 0.5
const JUMP_COYOTE_TIME = 0.10
const JUMP_BUFFER_TIME = 0.1
const WALLJUMP_EFFECT_STRENGTH = 0.1
const WALLJUMP_EFFECT_TIME = 0.15
const WALLJUMP_LOCK_TIME = 0.1
const WALLJUMP_X_RELATIVE_STRENGTH = 0.8
const DASH_TIME = 0.15
const DASH_VELOCITY = 350
const DASH_EFFECT_PERIOD = 0.02
const WALLJUMP_DETECT_DISTANCE = 8
const CORNER_CORRECTION_AMOUNT = 5
const DASHJUMP_COOLDOWN = 0.075 # Time after dashing before touching ground resets dash
const DASH_AMOUNT = 1

# Get the gravity from the project settings to be synced with RigidBody nodes.
var gravity = ProjectSettings.get_setting("physics/2d/default_gravity")
var facingDir = 1
var gravityReduction = 1
var lastOnGround = 0
var tryJumpedAt = 0
var wallJumpedAt = 0
var dashedAt = 0
var isJumping = false
var justLanded = false
var landingSpeed = Vector2()
var dashDir = Vector2()
var dashVelocity = DASH_VELOCITY
var dashGhost = preload("res://DashEffect.tscn")
var lastGhostedAt = 0
var dashes = DASH_AMOUNT
var lastVelocity = Vector2()
var lastActionAt = 0
var isDashing
var isClimbing
var isSliding
var climbStamina = CLIMB_STAMINA

var hairColors = [
	[Color("5779ec"), Color("364db0")],
	[Color("ac3232"), Color("b0373f")],
	[Color("7bf779"), Color("37b043")],
]

@onready var ani = get_node("ani")
@onready var initialPosition = position
@onready var initialVelocity = velocity
func _ready():
	reset()

func reset():
	velocity = initialVelocity
	position = initialPosition
	dashes = DASH_AMOUNT
	isDashing = false
	facingDir = 1
	gravityReduction = 1
	lastOnGround = 0
	tryJumpedAt = 0
	wallJumpedAt = 0
	dashedAt = 0
	isJumping = false
	justLanded = false


func _process(delta):
	ani.set_flip_h(facingDir < 0)
	if isClimbing:
		ani.play("climbing")
		if velocity.y == 0:
			ani.pause()
	elif isDashing:
		ani.play("dash")
	elif isSliding:
		ani.play("sliding")
	elif abs(velocity.x) != 0:
		ani.play("walk")
	else:
		ani.play("idle")

	ani.get_material().set_shader_parameter("whiteout", isClimbing && climbStamina < CLIMB_STAMINA / 3 && int(now() * 20) % 5 == 1)
	ani.get_material().set_shader_parameter("exhustion", (CLIMB_STAMINA - climbStamina) / CLIMB_STAMINA)
	ani.get_material().set_shader_parameter("replace_0", hairColors[dashes][0])
	ani.get_material().set_shader_parameter("replace_1", hairColors[dashes][1])


	if isJumping:
		ani.scale.y = lerpf(0.85, 1.2, abs(velocity.y) / abs(TERMINAL_VELOCITY))
		ani.scale.x = lerpf(1.15, 0.85, abs(velocity.y) / abs(TERMINAL_VELOCITY))
	elif justLanded:
		ani.scale.y = lerpf(1, 0.75, abs(landingSpeed.y) / abs(TERMINAL_VELOCITY))
		ani.scale.x = lerpf(1, 1.5, abs(landingSpeed.y) / abs(TERMINAL_VELOCITY))
	elif isDashing:
		ani.scale.y = 1
		ani.scale.x = 1
	else:
		ani.scale.y = lerpf(ani.scale.y, 1, 1-pow(0.001, delta))
		ani.scale.x = lerpf(ani.scale.x, 1, 1-pow(0.001, delta))

	$DashParticles.emitting = isDashing

	var col = $CameraLimitTest.get_collider()
	$Camera2D.limit_bottom = 100000
	if col != null:
		$Camera2D.limit_bottom = col.get_parent().position.y



func _physics_process(delta):
	isDashing = withinGrace(dashedAt, DASH_TIME)
	# Gravity
	if not is_on_floor():
		velocity.y += gravity * gravityReduction * delta
		landingSpeed = velocity


	# Wall Slide
	isSliding = false
	if on_wall() and velocity.y > 0:
		isSliding = true
		velocity.y *= 0.8

	# Climb
	if isClimbing and not on_wall():
		velocity.x = facingDir * SPEED
	isClimbing = false
	if on_wall() and Input.is_action_pressed("climb") and climbStamina > 0:
		climbStamina -= delta
		isClimbing = true
		velocity.y = 0
		var direction = Input.get_axis("move_up", "move_down")
		velocity.y = move_toward(velocity.y, direction * CLIMBSPEED, CLIMBSPEED)

	# Wiggle
	manageWiggle($wiggleRight, 1)
	manageWiggle($wiggleLeft, -1)

	# Handle Jump.
	if (Input.is_action_just_pressed("jump") or withinGrace(tryJumpedAt, JUMP_BUFFER_TIME)):
		if is_on_floor() or withinGrace(lastOnGround, JUMP_COYOTE_TIME) and not withinGrace(lastActionAt, DASHJUMP_COOLDOWN):
			jump(1)
		elif on_wall() and not withinGrace(lastActionAt, DASHJUMP_COOLDOWN):
			lastActionAt = now()
			velocity.y = JUMP_VELOCITY
			velocity.x = JUMP_VELOCITY * facingDir * WALLJUMP_X_RELATIVE_STRENGTH
			facingDir *= -1
			tryJumpedAt = 0
			wallJumpedAt = now()
			isJumping = true
			if isDashing:
				dashDir.y -= 0.7
				dashDir.x += 0.5 * facingDir
		else:
			if not withinGrace(tryJumpedAt, JUMP_BUFFER_TIME):
				tryJumpedAt = now()

	if not Input.is_action_pressed("jump") or velocity.y > 0:
		if velocity.y < 0 and isJumping:
			velocity.y *= JUMP_RELEASE_REDUCTION
			isJumping = false
		gravityReduction = 1

	justLanded = false
	if is_on_floor():
		if not withinGrace(lastActionAt, DASHJUMP_COOLDOWN):
			dashes = DASH_AMOUNT
		lastOnGround = now()
		climbStamina = CLIMB_STAMINA
		if not justLanded and isJumping:
			justLanded = true
		isJumping = false

	if Input.is_action_just_pressed("dash") and dashes > 0 and not withinGrace(lastActionAt, DASHJUMP_COOLDOWN):
		lastActionAt = now()
		dashes -= 1
		dashedAt = now()
		dashDir = Vector2(facingDir, 0)
		dashVelocity = DASH_VELOCITY
		addDashEffect()
		if Input.is_action_pressed("move_up"):
			dashDir = Vector2(0, -1)
		elif Input.is_action_pressed("move_down") and not is_on_floor():
			dashDir = Vector2(0, 1)

		if Input.is_action_pressed("move_left"):
			dashDir.x = -1
		elif Input.is_action_pressed("move_right"):
			dashDir.x = 1
		dashDir = dashDir.normalized()


	velocity.y = clampf(velocity.y, -TERMINAL_VELOCITY, TERMINAL_VELOCITY)


	var direction = Input.get_axis("move_left", "move_right")

	get_node("walldetecthead").target_position.x = WALLJUMP_DETECT_DISTANCE * facingDir
	get_node("walldetectfeet").target_position.x = WALLJUMP_DETECT_DISTANCE * facingDir

	# Touch Spikes
	for i in get_slide_collision_count():
		var collision = get_slide_collision(i)
		var col = collision.get_collider()
		if is_instance_of(col, TileMap):
			var rid = collision.get_collider_rid()
			var pos = col.get_coords_for_body_rid(rid)
			var data = col.get_cell_tile_data(1, pos)
			if data != null:
				var deadly = data.get_custom_data("deadly")
				if deadly:
					spike()


	if withinGrace(dashedAt, DASH_TIME):
		addDashEffect()
		velocity = dashDir * DASH_VELOCITY
	else:
		var speed = ACCEL
		if withinGrace(wallJumpedAt, WALLJUMP_EFFECT_TIME):
			speed *= WALLJUMP_EFFECT_STRENGTH
		if not withinGrace(wallJumpedAt, WALLJUMP_LOCK_TIME):
			if direction:
				if !(sign(velocity.x) == sign(direction) && SPEED < abs(velocity.x)):
					velocity.x = move_toward(velocity.x, direction * SPEED, speed)
				facingDir = sign(velocity.x)
			else:
				velocity.x = move_toward(velocity.x, 0, speed)

	if Input.is_action_just_pressed("kill"):
		kill()

	lastVelocity = velocity
	move_and_slide()

func manageWiggle(wiggler, delta):
	if wiggler.is_colliding() and velocity.y > 0:
		var og_pos = position.x
		var wiggleRoom = CORNER_CORRECTION_AMOUNT
		while wiggler.is_colliding() and wiggleRoom > 0:
			wiggleRoom -= 1
			position.x += delta
			wiggler.force_raycast_update()
		if wiggler.is_colliding():
			position.x = og_pos
		else:
			velocity = lastVelocity

func addDashEffect():
	if not withinGrace(lastGhostedAt, DASH_EFFECT_PERIOD):
		lastGhostedAt = now()
		var dash = dashGhost.instantiate()
		dash.scale.x = facingDir
		dash.position = position + ani.offset
		dash.get_material().set_shader_parameter("xOffset", dash.position.x)
		get_parent().get_parent().add_child(dash)

func on_wall():
	return is_collider_grabable($walldetectfeet) or is_collider_grabable($walldetecthead)

func is_collider_grabable(coll):
	if coll.is_colliding():
		var col = coll.get_collider()
		if col.name == "TileMap":
				var rid = coll.get_collider_rid()
				var pos = col.get_coords_for_body_rid(rid)
				var data = col.get_cell_tile_data(1, pos)
				if data != null:
					var deadly = data.get_custom_data("deadly")
					if deadly:
						return false
		return true
	return false

func withinGrace(time, period):
	return time + period > now()
func now():
	return Time.get_ticks_msec() / 1000.0 + 10

func jump(strengthMod):
	lastActionAt = now()
	velocity.y += JUMP_VELOCITY * strengthMod
	gravityReduction = JUMP_GRAVITY_REDUCTION
	tryJumpedAt = 0
	isJumping = true
	if isDashing:
		dashDir.y -= 0.5
		dashDir.x *= 1.5

func kill():
	# TODO: Add animation
	reset()


func spring():
	isDashing = false
	dashedAt = 0
	jump(2.5)
	isJumping = false
	dashes = DASH_AMOUNT
	gravityReduction = 1

func spike():
	kill()

var activeCheckpoint = null
func checkpoint(check):
	initialPosition = position
	if activeCheckpoint != null:
		activeCheckpoint.deactivate()
	activeCheckpoint = check
